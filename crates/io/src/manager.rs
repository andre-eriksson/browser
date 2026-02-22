use std::sync::Arc;

use constants::{
    events::{EVENT_ASSET_LOADED, EVENT_LOAD_ASSET},
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
    embeded::EmbededType,
    errors::AssetError,
    loader::{Loader, Writer},
};

/// AssetType represents the type of asset being managed by the AssetManager.
/// It can be an icon, font, or image.
#[derive(Debug)]
pub enum ResourceType<'a> {
    /// Represents resources that are embedded within the application, such as icons, fonts, images, shaders, browser assets, and root assets.
    Embeded(EmbededType<'a>),

    /// Represents resources that are stored in a cache, such as "$HOME/.cache/<app>/app_cache" on Unix-like systems.
    Cache(&'a str),

    /// Represents configuration files or settings, such as "$HOME/.config/<app>/app_config.json" on Unix-like systems.
    Config(&'a str),

    /// Represents user-generated data or preferences, such as "$HOME/.local/share/<app>/user_data.json" on Unix-like systems.
    UserData(&'a str),

    /// Any resource that can be accessed via a file path, such as "assets/image.png" or "/usr/local/data/file.txt".
    FileSystem(&'a str),

    /// Load any resource from an absolute path, given a protocol such as "file://", "http://", or "https://".
    /// The location field specifies the path or URL to the resource.
    Absolute {
        protocol: &'a str,
        location: &'a str,
    },
}

impl ResourceType<'_> {
    pub fn key(&self) -> String {
        match self {
            ResourceType::Embeded(embeded) => embeded.path(),
            ResourceType::Cache(path) => path.to_string(),
            ResourceType::Config(path) => path.to_string(),
            ResourceType::UserData(path) => path.to_string(),
            ResourceType::FileSystem(path) => path.to_string(),
            ResourceType::Absolute { location, .. } => location.to_string(),
        }
    }
}

/// AssetManager is responsible for managing and loading assets from various backends.
pub struct Resource;

impl Resource {
    /// Fetches a resource from a remote URL, applying the necessary policies and handling cookies and headers.
    #[cfg(feature = "network")]
    pub async fn from_remote<'a>(
        url: &'a str,
        client: &'a dyn HttpClient,
        cookies: &'a [Cookie],
        browser_headers: &'a Arc<HeaderMap>,
        page_url: &'a Option<Url>,
        policies: &'a DocumentPolicy,
    ) -> Result<Box<dyn ResponseHandle>, RequestError> {
        let url = match page_url.as_ref() {
            Some(u) => u.join(url),
            None => Url::parse(url),
        }
        .map_err(|e| RequestError::Network(NetworkError::InvalidUrl(e.to_string())))?;

        let request = RequestBuilder::from(url).build();
        let mut service = NetworkService::new(client, cookies, browser_headers);
        let header_response = match service.fetch(page_url.clone(), policies, request).await {
            RequestResult::Failed(err) => return Err(err),
            RequestResult::ClientError(resp)
            | RequestResult::ServerError(resp)
            | RequestResult::Success(resp) => resp,
        };

        Ok(header_response)
    }

    /// Loads an asset from the specified resource type, handling both embedded and filesystem resources.
    pub fn load(resource: ResourceType) -> Result<Vec<u8>, AssetError> {
        let path = resource.key();
        trace!({ EVENT } = EVENT_LOAD_ASSET);

        if let Ok(data) = resource.load_asset() {
            trace!({ EVENT } = EVENT_ASSET_LOADED);

            return Ok(data);
        }

        Err(AssetError::NotFound(path))
    }

    /// Writes data to a specified resource, such as cache or config files.
    /// This operation is not supported for embedded or absolute resources.
    pub fn write<C>(resource: ResourceType, data: C) -> Result<(), AssetError>
    where
        C: AsRef<[u8]>,
    {
        resource.write(data)
    }

    /// Loads an embedded asset directly, bypassing the need for a ResourceType wrapper.
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
