use std::collections::HashMap;

use constants::{
    events::{EVENT_ASSET_CACHE_HIT, EVENT_ASSET_LOADED, EVENT_ASSET_NOT_FOUND, EVENT_LOAD_ASSET},
    keys::EVENT,
};
use tracing::{instrument, trace, warn};

use crate::{
    backends::{AsyncBackend, Backend, SyncBackend},
    errors::AssetError,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmbededAsset<'a> {
    Icon(&'a str),
    Font(&'a str),
    Image(&'a str),
    Shader(&'a str),
    Browser(&'a str),
}

impl EmbededAsset<'_> {
    pub fn path(&self) -> String {
        match self {
            EmbededAsset::Icon(name) => format!("icon/{}", name),
            EmbededAsset::Font(name) => format!("font/{}", name),
            EmbededAsset::Image(name) => format!("image/{}", name),
            EmbededAsset::Shader(name) => format!("shader/{}", name),
            EmbededAsset::Browser(name) => format!("browser/{}", name),
        }
    }
}

/// AssetType represents the type of asset being managed by the AssetManager.
/// It can be an icon, font, or image.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType<'a> {
    FileSystem(&'a str),
    Remote(&'a str),
    /// embed://, file://, http://, https://, etc.
    Absolute {
        protocol: &'a str,
        location: &'a str,
    },
}

impl ResourceType<'_> {
    pub fn resolve_absolute(&'_ self) -> ResourceType<'_> {
        match self {
            ResourceType::Absolute { protocol, location } => match *protocol {
                "file" => ResourceType::FileSystem(location),
                _ => ResourceType::Remote(location),
            },
            other => other.clone(),
        }
    }

    pub fn path(&self) -> String {
        match self {
            ResourceType::FileSystem(path) => path.to_string(),
            ResourceType::Remote(url) => url.to_string(),
            ResourceType::Absolute { location, .. } => location.to_string(),
        }
    }
}

/// AssetManager is responsible for managing and loading assets from various backends.
pub struct Resource;

impl Resource {
    #[instrument(fields(resource = ?resource))]
    pub async fn load_async<'a>(
        resource: ResourceType<'_>,
        cache: &mut HashMap<String, Vec<u8>>,
        backends: Vec<Backend<'a>>,
    ) -> Result<Vec<u8>, AssetError> {
        let key = resource.resolve_absolute().path();

        if let Some(cached) = cache.get(&key) {
            trace!({ EVENT } = EVENT_ASSET_CACHE_HIT);

            return Ok(cached.clone());
        }

        for backend in backends {
            if let Ok(data) = backend.await_asset(&key).await {
                trace!({ EVENT } = EVENT_ASSET_LOADED);

                cache.insert(key.clone(), data.clone());

                return Ok(data);
            }
        }

        trace!({ EVENT } = EVENT_ASSET_NOT_FOUND);

        Err(AssetError::NotFound(key))
    }

    pub fn load_sync(resource: ResourceType<'_>, backends: Vec<Backend<'_>>) -> Vec<u8> {
        let key = if let ResourceType::Absolute { protocol, location } = resource {
            format!("{}://{}", protocol, location)
        } else {
            resource.resolve_absolute().path()
        };

        for backend in backends {
            if let Ok(data) = backend.load_asset(&key) {
                trace!({ EVENT } = EVENT_ASSET_LOADED);

                return data;
            }
        }

        trace!({ EVENT } = EVENT_ASSET_NOT_FOUND);

        panic!("Asset not found: {}", key);
    }

    #[instrument(fields(embeded_asset = ?embeded_asset))]
    pub fn load_embedded(embeded_asset: EmbededAsset) -> Vec<u8> {
        let location = embeded_asset.path();

        trace!({ EVENT } = EVENT_LOAD_ASSET);

        if let Ok(data) = Backend::Embedded.load_asset(&location) {
            trace!({ EVENT } = EVENT_ASSET_LOADED);

            return data;
        }

        panic!("Embedded asset not found: {}", location);
    }
}
