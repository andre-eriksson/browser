use std::{collections::HashMap, path::PathBuf};

use api::logging::{
    EVENT, EVENT_ASSET_CACHE_HIT, EVENT_ASSET_LOADED, EVENT_ASSET_NOT_FOUND, EVENT_LOAD_ASSET,
};
use tracing::{debug, info, warn};

use crate::backends::{AssetBackend, Backend};

/// AssetType represents the type of asset being managed by the AssetManager.
/// It can be an icon, font, or image.
///
/// # Variants
/// * `Icon` - Represents an icon asset, identified by its name.
/// * `Font` - Represents a font asset, identified by its name.
/// * `Image` - Represents an image asset, identified by its name.
pub enum AssetType {
    Icon(&'static str),
    Font(&'static str),
    Image(&'static str),
}

/// AssetManager is responsible for managing and loading assets from various backends.
///
/// # Fields
/// * `backends` - A vector of backends to load assets from, in order of priority.
/// * `cache` - A cache to store loaded assets to avoid redundant loading.
pub struct AssetManager {
    backends: Vec<Backend>,
    cache: HashMap<String, Vec<u8>>,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self {
            backends: vec![
                Backend::FileSystem(PathBuf::from("assets")),
                Backend::Embedded,
            ],
            cache: HashMap::new(),
        }
    }
}

impl AssetManager {
    /// Creates a new AssetManager with the specified backends.
    ///
    /// # Arguments
    /// * `backends` - A vector of backends to use for loading assets, will prioritize the first one.
    pub fn new(backends: Vec<Backend>) -> Self {
        Self {
            backends,
            cache: HashMap::new(),
        }
    }

    /// Retrieves an asset by its type.
    ///
    /// # Arguments
    /// * `asset` - The type of asset to retrieve, which can be an icon, font, or image.
    ///
    /// # Returns
    /// A vector of bytes representing the asset data.
    ///
    /// # Panics
    /// If the asset cannot be found in any backend and the fallback asset is also not available.
    pub fn get(&mut self, asset: AssetType) -> Vec<u8> {
        let key = match asset {
            AssetType::Icon(name) => format!("icon/{}", name),
            AssetType::Font(name) => format!("font/{}", name),
            AssetType::Image(name) => format!("image/{}", name),
        };

        debug!({ EVENT } = EVENT_LOAD_ASSET, key);

        if let Some(data) = self.cache.get(&key) {
            debug!({ EVENT } = EVENT_ASSET_CACHE_HIT, key);
            return data.clone();
        }

        for backend in &self.backends {
            if let Some(data) = backend.load_asset(&key) {
                info!({ EVENT } = EVENT_ASSET_LOADED, "{}", key.clone());

                self.cache.insert(key, data.clone());
                return data;
            }
        }

        warn!({ EVENT } = EVENT_ASSET_NOT_FOUND, key);

        let fallback = Backend::Embedded.load_asset("fallback.png");

        if let Some(fallback_data) = fallback {
            self.cache.insert(key, fallback_data.clone());
            return fallback_data;
        }

        panic!("Fallback asset not found: {}", key);
    }
}
