use std::{fs::File, io::Read, path::PathBuf};

use cookies::Cookie;
use network::{
    HeaderMap,
    client::{HttpClient, ResponseHandle},
    errors::{NetworkError, RequestError},
    request::RequestBuilder,
};
use storage::Directory;
use tracing::{instrument, trace, warn};
use url::Url;

use crate::{DocumentPolicy, HttpCache, RequestResult, files::FilePath, network::request::NetworkService};

use crate::{
    Entry,
    embeded::EmbededType,
    errors::ResourceError,
    loader::{Loader, Writer},
};

/// `AssetType` represents the type of asset being managed by the `AssetManager`.
/// It can be an icon, font, or image.
#[derive(Debug, Clone)]
pub enum ResourceType<'resource> {
    /// Represents resources that are embedded within the application, such as icons, fonts, images, shaders, browser assets, and root assets.
    Embeded(EmbededType<'resource>),

    /// Any resource that can be accessed via a file path, such as "assets/image.png" or "/usr/local/data/file.txt".
    Path(Entry<'resource>),

    /// Load any resource from an absolute path, given a protocol such as "file://", "http://", or "https://".
    /// The location field specifies the path or URL to the resource.
    Absolute {
        protocol: &'resource str,
        location: &'resource str,
    },
}

impl ResourceType<'_> {
    #[must_use]
    pub fn key(&self) -> String {
        match self {
            ResourceType::Embeded(embeded) => embeded.path(),
            ResourceType::Path(file) => file.location().to_string(),
            ResourceType::Absolute { location, .. } => location.to_string(),
        }
    }
}

/// `AssetManager` is responsible for managing and loading assets from various backends.
pub struct Resource;

impl Resource {
    /// Default maximum file size limit for loading resources, set to 10 MiB. This limit helps prevent excessive memory usage when loading large files.
    pub const DEFAULT_MAX_FILE_SIZE: Option<u64> = Some(10 * 1024 * 1024);

    /// Default maximum number of files to load from a directory, set to 100. This limit helps prevent performance issues when loading directories with a large number of files.
    pub const DEFAULT_MAX_FILES: Option<u64> = Some(100);

    /// Fetches a resource from a remote URL, applying the necessary policies and handling cookies and headers.
    #[allow(clippy::too_many_arguments)]
    pub async fn from_remote<'app>(
        dirs: Directory,
        url: &'app str,
        cache: &HttpCache,
        client: &'app dyn HttpClient,
        cookies: &'app [Cookie],
        browser_headers: &'app HeaderMap,
        page_url: Option<Url>,
        policies: &'app DocumentPolicy,
    ) -> Result<Box<dyn ResponseHandle>, RequestError> {
        let url = page_url
            .as_ref()
            .map_or_else(|| Url::parse(url), |u| u.join(url))
            .map_err(|error| RequestError::Network(NetworkError::InvalidUrl(error)))?;

        let request = RequestBuilder::from(url).build();
        let service = NetworkService::new(client, cookies, browser_headers);
        let header_response = match service
            .fetch(dirs, cache, page_url.clone(), policies, request)
            .await
        {
            RequestResult::Failed(err) => return Err(err),
            RequestResult::ClientError(resp) | RequestResult::ServerError(resp) | RequestResult::Success(resp) => resp,
        };

        Ok(header_response)
    }

    /// Loads an asset from the specified resource type, handling both embedded and filesystem resources.
    ///
    /// # Args
    /// * `resource` - The resource to load, which can be an embedded asset, a file path, or an absolute URL.
    /// * `max_file_size` - An optional maximum file size limit in bytes. If the loaded asset exceeds this size, an error will be returned. If `None`, there is no size limit.
    #[instrument(fields(resource = ?resource.key()))]
    pub fn load(resource: ResourceType, dirs: Directory, max_file_size: Option<u64>) -> Result<Vec<u8>, ResourceError> {
        match resource.load_asset(Some(dirs), max_file_size) {
            Ok(data) => {
                trace!("OK");
                Ok(data)
            }
            Err(error) => {
                trace!(%error);
                Err(error)
            }
        }
    }

    /// Loads all files from a specified directory resource, returning their contents as a vector of byte vectors.
    ///
    /// # Args
    /// * `dir` - The directory resource to load, which should be a `ResourceType::Path` pointing to a directory.
    /// * `max_files` - An optional maximum number of files to load from the directory. If the directory contains more files than this limit, an error will be returned.
    ///   If `None`, there is no limit on the number of files.
    /// * `max_file_size` - An optional maximum file size limit in bytes for each file. If any loaded file exceeds this size, it will be skipped and a warning will be logged.
    ///   If `None`, there is no size limit for individual files.
    pub fn load_dir(
        dir: Entry,
        dirs: &Directory,
        max_files: Option<usize>,
        max_file_size: Option<u64>,
    ) -> Result<Vec<Vec<u8>>, ResourceError> {
        let path = if dir.is_global() {
            match dir.file_path() {
                FilePath::Absolute => PathBuf::from(dir.location()),
                FilePath::Cache => dirs.global_cache.join(dir.location()),
                FilePath::Config => dirs.global_config.join(dir.location()),
                FilePath::UserData => dirs.global_data.join(dir.location()),
                FilePath::Temporary => dirs.temp.join(dir.location()),
            }
        } else {
            match dir.file_path() {
                FilePath::Absolute => PathBuf::from(dir.location()),
                FilePath::Cache => dirs.profile_cache.join(dir.location()),
                FilePath::Config => dirs.profile_config.join(dir.location()),
                FilePath::UserData => dirs.profile_data.join(dir.location()),
                FilePath::Temporary => dirs.temp.join(dir.location()),
            }
        };

        let mut paths = Vec::new();

        for entry in
            std::fs::read_dir(path).map_err(|_| ResourceError::NotFound("Directory doesn't exist".to_string()))?
        {
            if let Some(max) = max_files
                && paths.len() >= max
            {
                return Err(ResourceError::TooManyEntries(format!(
                    "Directory contains too many entries, which exceeds the limit of {max}"
                )));
            }

            let entry = entry.map_err(|_| ResourceError::NotFound("Entry doesn't exist".to_string()))?;
            paths.push(entry.path());
        }

        let mut files = Vec::new();

        for path in paths {
            let Ok(mut file) = File::open(path) else {
                continue;
            };

            let Ok(metadata) = file.metadata() else {
                continue;
            };

            if !metadata.is_file() {
                continue;
            }

            if let Some(max) = max_file_size
                && metadata.len() > max
            {
                return Err(ResourceError::FileTooLarge {
                    data_size: metadata.len(),
                    max_size: max,
                });
            }

            let mut buffer = Vec::with_capacity(metadata.len() as usize);
            file.read_to_end(&mut buffer)
                .map_err(|e| ResourceError::Io(e.to_string()))?;
            files.push(buffer);
        }

        Ok(files)
    }

    /// Writes data to a specified resource, such as cache or config files.
    /// This operation is not supported for embedded or absolute resources.
    pub fn write<C>(resource: ResourceType, data: C, dirs: Directory) -> Result<(), ResourceError>
    where
        C: AsRef<[u8]>,
    {
        resource.write(data, dirs)
    }

    /// Loads an embedded asset directly, which is useful for resources that are compiled into the application binary,
    /// such as icons, fonts, and shaders.
    ///
    /// # Panics
    /// If the embedded asset cannot be found, which should not happen if the asset is correctly included in the build.
    #[instrument(fields(?asset))]
    pub fn load_embedded(asset: EmbededType) -> Vec<u8> {
        let path = &asset.path();

        if let Ok(data) = ResourceType::Embeded(asset).load_asset(None, None) {
            trace!("OK");

            return data;
        }

        panic!("Embedded asset not found: {}", path);
    }
}
