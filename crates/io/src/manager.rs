use constants::{
    events::{EVENT_ASSET_LOADED, EVENT_ASSET_NOT_FOUND, EVENT_LOAD_ASSET},
    keys::EVENT,
};
use cookies::Cookie;
#[cfg(feature = "network")]
use network::{
    HeaderMap,
    client::{HttpClient, ResponseHandle},
    errors::{NetworkError, RequestError},
    request::RequestBuilder,
};
use tracing::{instrument, trace, warn};
#[cfg(feature = "network")]
use url::Url;

#[cfg(feature = "network")]
use crate::{DocumentPolicy, RequestResult, network::request::NetworkService};

use crate::{
    Entry,
    embeded::EmbededType,
    errors::ResourceError,
    loader::{Loader, Writer},
};

/// AssetType represents the type of asset being managed by the AssetManager.
/// It can be an icon, font, or image.
#[derive(Debug)]
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
    pub fn key(&self) -> String {
        match self {
            ResourceType::Embeded(embeded) => embeded.path(),
            ResourceType::Path(file) => file
                .path()
                .map_or_else(|| file.location().to_string(), |p| p.to_string_lossy().to_string()),
            ResourceType::Absolute { location, .. } => location.to_string(),
        }
    }
}

/// AssetManager is responsible for managing and loading assets from various backends.
pub struct Resource;

impl Resource {
    /// Fetches a resource from a remote URL, applying the necessary policies and handling cookies and headers.
    #[cfg(feature = "network")]
    pub async fn from_remote<'app>(
        url: &'app str,
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
        let header_response = match service.fetch(page_url.clone(), policies, request).await {
            RequestResult::Failed(err) => return Err(err),
            RequestResult::ClientError(resp) | RequestResult::ServerError(resp) | RequestResult::Success(resp) => resp,
        };

        Ok(header_response)
    }

    /// Loads an asset from the specified resource type, handling both embedded and filesystem resources.
    pub fn load(resource: ResourceType) -> Result<Vec<u8>, ResourceError> {
        let path = resource.key();
        trace!({ EVENT } = EVENT_LOAD_ASSET);

        if let Ok(data) = resource.load_asset() {
            trace!({ EVENT } = EVENT_ASSET_LOADED);

            return Ok(data);
        }

        trace!({ EVENT } = EVENT_ASSET_NOT_FOUND);
        Err(ResourceError::NotFound(path))
    }

    pub fn load_dir(dir: Entry) -> Result<Vec<Vec<u8>>, ResourceError> {
        if let Some(path) = dir.path() {
            let mut result = Vec::new();
            for entry in
                std::fs::read_dir(path).map_err(|_| ResourceError::NotFound("Directory doesn't exist".to_string()))?
            {
                let entry = entry.map_err(|_| ResourceError::NotFound("Entry doesn't exist".to_string()))?;
                let path = entry.path();

                if path.is_file()
                    && let Ok(data) = std::fs::read(&path)
                {
                    result.push(data);
                }
            }

            Ok(result)
        } else {
            Err(ResourceError::NotFound("Directory path is unavailable".to_string()))
        }
    }

    /// Writes data to a specified resource, such as cache or config files.
    /// This operation is not supported for embedded or absolute resources.
    pub fn write<C>(resource: ResourceType, data: C) -> Result<(), ResourceError>
    where
        C: AsRef<[u8]>,
    {
        resource.write(data)
    }

    /// Loads an embedded asset directly, which is useful for resources that are compiled into the application binary,
    /// such as icons, fonts, and shaders.
    ///
    /// # Panics
    /// If the embedded asset cannot be found, which should not happen if the asset is correctly included in the build.
    #[instrument(fields(embeded_asset = ?embeded_asset))]
    pub fn load_embedded(embeded_asset: EmbededType) -> Vec<u8> {
        let path = &embeded_asset.path();
        trace!({ EVENT } = EVENT_LOAD_ASSET);

        if let Ok(data) = ResourceType::Embeded(embeded_asset).load_asset() {
            trace!({ EVENT } = EVENT_ASSET_LOADED);

            return data;
        }

        panic!("Embedded asset not found: {}", path);
    }
}
