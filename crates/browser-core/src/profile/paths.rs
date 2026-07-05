use std::{path::PathBuf, sync::Arc};

use rand::{RngExt, distr::Alphanumeric};
use storage::{Directory, get_cache_path, get_config_path, get_data_path, get_temp_path};

use crate::profile::ProfileKind;

#[derive(Debug)]
pub struct ProfilePaths {
    cache: Arc<PathBuf>,
    config: Arc<PathBuf>,
    data: Arc<PathBuf>,
    temp: Arc<PathBuf>,
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
                    cache: Arc::new(cache.unwrap_or_else(|| stable_temp.join(&id).join(names.0))),
                    config: Arc::new(config.unwrap_or_else(|| stable_temp.join(&id).join(names.1))),
                    data: Arc::new(data.unwrap_or_else(|| stable_temp.join(&id).join(names.2))),
                    temp: Arc::new(stable_temp.join(&id)),
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
                    cache: Arc::new(session_temp.join(names.0)),
                    config: Arc::new(session_temp.join(names.1)),
                    data: Arc::new(session_temp.join(names.2)),
                    temp: Arc::new(session_temp),
                    degraded: false,
                    is_temporary: true,
                }
            }
        }
    }

    pub fn cache(&self) -> Arc<PathBuf> {
        Arc::clone(&self.cache)
    }

    pub fn config(&self) -> Arc<PathBuf> {
        Arc::clone(&self.config)
    }

    pub fn data(&self) -> Arc<PathBuf> {
        Arc::clone(&self.data)
    }

    pub fn temp(&self) -> Arc<PathBuf> {
        Arc::clone(&self.temp)
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
            let _ = std::fs::remove_dir_all(&*self.cache);
            let _ = std::fs::remove_dir_all(&*self.config);
            let _ = std::fs::remove_dir_all(&*self.data);
        }
    }
}

impl From<&ProfilePaths> for Directory {
    fn from(value: &ProfilePaths) -> Self {
        Directory {
            cache: Arc::clone(&value.cache),
            config: Arc::clone(&value.config),
            data: Arc::clone(&value.data),
            temp: Arc::clone(&value.temp),
        }
    }
}
