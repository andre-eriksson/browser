use std::{collections::HashMap, path::PathBuf};

use api::logging::{
    EVENT, EVENT_ASSET_CACHE_HIT, EVENT_ASSET_LOADED, EVENT_ASSET_NOT_FOUND, EVENT_LOAD_ASSET,
};
use tracing::{debug, info, warn};

use crate::backends::{AssetBackend, Backend};

pub enum AssetType {
    Icon(&'static str),
    Font(&'static str),
    Image(&'static str),
}

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
    pub fn new(backends: Vec<Backend>) -> Self {
        Self {
            backends,
            cache: HashMap::new(),
        }
    }

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
