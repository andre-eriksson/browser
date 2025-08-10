use std::{collections::HashMap, path::PathBuf};

use tracing::{debug, info, warn};

use api::logging::{
    EVENT, EVENT_ASSET_CACHE_HIT, EVENT_ASSET_LOADED, EVENT_ASSET_NOT_FOUND, EVENT_LOAD_ASSET,
};

use crate::backends::{AssetBackend, AssetError, Backend};

/// AssetType represents the type of asset being managed by the AssetManager.
/// It can be an icon, font, or image.
pub enum AssetType {
    /// Represents an icon asset, default path would be `"icon/{name}"`.j
    Icon(&'static str),

    /// Represents a font asset, default path would be `"font/{name}"`.
    Font(&'static str),

    /// Represents an image asset, default path would be `"image/{name}"`.
    Image(&'static str),
}

/// AssetManager is responsible for managing and loading assets from various backends.
pub struct AssetManager {
    /// A vector of backends to load assets from, in order of priority.
    backends: Vec<Backend>,

    /// A cache to store loaded assets to avoid redundant loading.
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

    /// Loads an asset, testing the suppplied backends in order until the asset is found.
    /// If an asset is found it is stored in a in-memory cache.
    ///
    /// # Arguments
    /// * `asset` - The type of asset to retrieve.
    ///
    /// # Returns
    /// A `Result<Vec<u8>, AssetError>` representing the asset data or an error message.
    pub fn load(&mut self, asset: AssetType) -> Result<Vec<u8>, AssetError> {
        let key = match asset {
            AssetType::Icon(name) => format!("icon/{}", name),
            AssetType::Font(name) => format!("font/{}", name),
            AssetType::Image(name) => format!("image/{}", name),
        };

        debug!({ EVENT } = EVENT_LOAD_ASSET, key);

        if let Some(data) = self.cache.get(&key) {
            debug!({ EVENT } = EVENT_ASSET_CACHE_HIT, key);
            return Ok(data.clone());
        }

        for backend in &self.backends {
            match backend.load_asset(&key) {
                Ok(data) => {
                    info!({ EVENT } = EVENT_ASSET_LOADED, key);

                    self.cache.insert(key, data.clone());
                    return Ok(data);
                }
                Err(AssetError::NotFound(_)) => continue,
                Err(e) => return Err(e),
            }
        }

        warn!(
            { EVENT } = EVENT_ASSET_NOT_FOUND,
            "Asset not found: {}", key
        );

        Err(AssetError::NotFound(key))
    }

    /// Loads an embedded asset by its type, with either a guaranteed return value or a panic.
    ///
    /// # Arguments
    /// * `asset` - The type of asset to load.
    ///
    /// # Returns
    /// A vector of bytes representing the asset data.
    ///
    /// # Panics
    /// If the asset cannot be found in the embedded backend.
    pub fn load_embedded(&self, asset: AssetType) -> Vec<u8> {
        let key = match asset {
            AssetType::Icon(name) => format!("icon/{}", name),
            AssetType::Font(name) => format!("font/{}", name),
            AssetType::Image(name) => format!("image/{}", name),
        };

        debug!({ EVENT } = EVENT_LOAD_ASSET, key);

        if let Ok(data) = Backend::Embedded.load_asset(&key) {
            info!({ EVENT } = EVENT_ASSET_LOADED, "{}", key.clone());

            return data;
        }

        panic!("Embedded asset not found: {}", key);
    }
}
