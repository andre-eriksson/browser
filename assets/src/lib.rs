use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::{backends::Backend, manager::AssetManager};

pub mod backends;
pub mod constants;
pub mod manager;

pub static ASSETS: Lazy<Mutex<AssetManager>> =
    Lazy::new(|| Mutex::new(AssetManager::new(vec![Backend::Embedded])));
