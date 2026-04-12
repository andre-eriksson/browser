//! This module defines constants for file names used in the browser's cache and configuration directories.
//! These constants are of type `ResourceType`, which is an enum that categorizes resources based on their
//! storage location (cache, config, user data). The constants defined in this module provide a standardized
//! way to reference specific files used by the browser, such as user agent stylesheets and user preferences.

use std::path::PathBuf;

use storage::paths::{get_cache_path, get_config_path, get_data_path};
use tracing::warn;

use crate::ResourceType;

#[derive(Debug)]
pub enum FilePath {
    Cache,
    Config,
    UserData,
    Absolute,
}

#[derive(Debug)]
pub struct Entry<'path> {
    location: &'path str,
    file_path: FilePath,
}

impl<'path> Entry<'path> {
    pub fn location(&self) -> &'path str {
        self.location
    }

    pub const fn cache(path: &'path str) -> Self {
        Self {
            location: path,
            file_path: FilePath::Cache,
        }
    }

    pub const fn config(path: &'path str) -> Self {
        Self {
            location: path,
            file_path: FilePath::Config,
        }
    }

    pub const fn user_data(path: &'path str) -> Self {
        Self {
            location: path,
            file_path: FilePath::UserData,
        }
    }

    pub fn absolute(path: &'path str) -> Self {
        Self {
            location: path,
            file_path: FilePath::Absolute,
        }
    }

    pub fn path(&self) -> Option<PathBuf> {
        match self.file_path {
            FilePath::Cache => {
                let cache_path = get_cache_path();

                match cache_path {
                    Some(path) => Some(path.join(self.location)),
                    None => {
                        warn!("Cache directory is unavailable");
                        None
                    }
                }
            }
            FilePath::Config => {
                let config_path = get_config_path();

                match config_path {
                    Some(path) => Some(path.join(self.location)),
                    None => {
                        warn!("Config directory is unavailable");
                        None
                    }
                }
            }
            FilePath::UserData => {
                let user_data_path = get_data_path();

                match user_data_path {
                    Some(path) => Some(path.join(self.location)),
                    None => {
                        warn!("User data directory is unavailable");
                        None
                    }
                }
            }
            FilePath::Absolute => Some(PathBuf::from(self.location)),
        }
    }
}

/// The cache file name for user agent stylesheets.
/// This file is stored in the cache directory and contains precompiled stylesheets for user agent (browser default) styles.
pub const CACHE_USER_AGENT: ResourceType = ResourceType::Path(Entry::cache("stylesheets/useragent.bin"));

/// The user preferences file name. This file is stored in the config directory and contains user-specific settings for the browser.
pub const PREFERENCES: ResourceType = ResourceType::Path(Entry::config("preferences.toml"));
