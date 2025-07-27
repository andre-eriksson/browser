use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::{backends::Backend, manager::AssetManager};

pub mod backends;
pub mod constants;
pub mod manager;

/// ASSETS is a global instance of AssetManager initialized with the Embedded backend.
/// It is wrapped in a Mutex to allow safe concurrent access.
pub static ASSETS: Lazy<Mutex<AssetManager>> =
    Lazy::new(|| Mutex::new(AssetManager::new(vec![Backend::Embedded])));
