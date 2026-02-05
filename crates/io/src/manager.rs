use std::{collections::HashMap, sync::Arc};

use constants::{
    events::{EVENT_ASSET_CACHE_HIT, EVENT_ASSET_LOADED, EVENT_ASSET_NOT_FOUND, EVENT_LOAD_ASSET},
    keys::EVENT,
};
use cookies::CookieJar;
use network::{HeaderMap, client::HttpClient};
use tracing::{instrument, trace, warn};
use url::Url;

use crate::{
    DocumentPolicy,
    errors::AssetError,
    loader::{AsyncLoader, SyncLoader},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmbededType<'a> {
    Icon(&'a str),
    Font(&'a str),
    Image(&'a str),
    Shader(&'a str),
    Browser(&'a str),
    Root(&'a str),
}

impl EmbededType<'_> {
    pub fn path(&self) -> String {
        match self {
            EmbededType::Icon(name) => format!("icon/{}", name),
            EmbededType::Font(name) => format!("font/{}", name),
            EmbededType::Image(name) => format!("image/{}", name),
            EmbededType::Shader(name) => format!("shader/{}", name),
            EmbededType::Browser(name) => format!("browser/{}", name),
            EmbededType::Root(name) => name.to_string(),
        }
    }
}

/// AssetType represents the type of asset being managed by the AssetManager.
/// It can be an icon, font, or image.
#[derive(Debug)]
pub enum ResourceType<'a> {
    Embeded(EmbededType<'a>),
    FileSystem(&'a str),
    Remote {
        url: &'a str,
        client: &'a dyn HttpClient,
        cookie_jar: &'a mut CookieJar,
        browser_headers: &'a Arc<HeaderMap>,
        page_url: &'a Option<Url>,
        policies: &'a DocumentPolicy,
    },
    /// embed://, file://, http://, https://, etc.
    Absolute {
        protocol: &'a str,
        location: &'a str,
        client: &'a dyn HttpClient,
        cookie_jar: &'a mut CookieJar,
        browser_headers: &'a Arc<HeaderMap>,
        page_url: &'a Option<Url>,
        policies: &'a DocumentPolicy,
    },
}

impl ResourceType<'_> {
    pub fn key(&self) -> String {
        match self {
            ResourceType::Embeded(embeded) => embeded.path(),
            ResourceType::FileSystem(path) => path.to_string(),
            ResourceType::Remote { url, .. } => url.to_string(),
            ResourceType::Absolute { location, .. } => location.to_string(),
        }
    }
}

/// AssetManager is responsible for managing and loading assets from various backends.
pub struct Resource;

impl Resource {
    #[instrument(fields(resource = ?resource))]
    pub async fn load_async(
        resource: ResourceType<'_>,
        cache: &mut HashMap<String, Vec<u8>>,
    ) -> Result<Vec<u8>, AssetError> {
        let key = &resource.key();

        if let Some(cached) = cache.get(&resource.key()) {
            trace!({ EVENT } = EVENT_ASSET_CACHE_HIT);

            return Ok(cached.clone());
        }

        if let Ok(data) = resource.await_asset().await {
            trace!({ EVENT } = EVENT_ASSET_LOADED);

            cache.insert(key.clone(), data.clone());

            return Ok(data);
        }

        trace!({ EVENT } = EVENT_ASSET_NOT_FOUND);

        Err(AssetError::NotFound(key.clone()))
    }

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
