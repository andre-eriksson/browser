use std::sync::RwLock;

use once_cell::sync::Lazy;

use crate::manager::AssetManager;

/// The backends module contains implementations for different asset backends.
pub mod backends;

/// The constants module contains definitions for various asset constants for consistency when trying to load assets.
pub mod constants;

/// The asset manager module contains the AssetManager struct allowing for the management and loading of assets.
pub mod manager;

/// ASSETS is a global instance of AssetManager initialized with the Embedded backend.
pub static ASSETS: Lazy<RwLock<AssetManager>> = Lazy::new(|| RwLock::new(AssetManager::default()));
