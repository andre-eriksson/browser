use std::path::{Path, PathBuf};

use rand::{RngExt, distr::Alphanumeric};
use storage::{get_cache_path, get_config_path, get_data_path, get_temp_path};

use crate::profile::ProfileKind;

#[derive(Debug)]
pub struct ProfilePaths {
    cache: PathBuf,
    config: PathBuf,
    data: PathBuf,
    degraded: bool,
    is_temporary: bool,
}

impl ProfilePaths {
    pub fn new(kind: ProfileKind) -> Self {
        match kind {
            ProfileKind::Persistent { id } => {
                let id = id.unwrap_or_else(|| "default".to_string());
                let cache = get_cache_path().map(|p| p.join(&id));
                let config = get_config_path().map(|p| p.join(&id));
                let data = get_data_path().map(|p| p.join(&id));
                let degraded = cache.is_none() || config.is_none() || data.is_none();

                let stable_temp = get_temp_path(None);
                let names = Self::dir_names();

                Self {
                    cache: cache.unwrap_or_else(|| stable_temp.join(&id).join(names.0)),
                    config: config.unwrap_or_else(|| stable_temp.join(&id).join(names.1)),
                    data: data.unwrap_or_else(|| stable_temp.join(&id).join(names.2)),
                    degraded,
                    is_temporary: false,
                }
            }
            ProfileKind::Temporary => {
                let suffix: String = rand::rng()
                    .sample_iter(Alphanumeric)
                    .take(6)
                    .map(char::from)
                    .collect();

                let session_temp = get_temp_path(Some(&suffix));
                let names = Self::dir_names();

                Self {
                    cache: session_temp.join(names.0),
                    config: session_temp.join(names.1),
                    data: session_temp.join(names.2),
                    degraded: false,
                    is_temporary: true,
                }
            }
        }
    }

    pub fn cache(&self) -> &Path {
        self.cache.as_path()
    }

    pub fn config(&self) -> &Path {
        self.config.as_path()
    }

    pub fn data(&self) -> &Path {
        self.data.as_path()
    }

    pub fn is_degraded(&self) -> bool {
        self.degraded
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn dir_names() -> (&'static str, &'static str, &'static str) {
        ("Cache", "Config", "Data")
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn dir_names() -> (&'static str, &'static str, &'static str) {
        ("cache", "config", "data")
    }
}

impl Drop for ProfilePaths {
    fn drop(&mut self) {
        if self.is_temporary {
            let _ = std::fs::remove_dir_all(&self.cache);
            let _ = std::fs::remove_dir_all(&self.config);
            let _ = std::fs::remove_dir_all(&self.data);
        }
    }
}
