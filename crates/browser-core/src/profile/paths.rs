use std::{path::PathBuf, sync::Arc};

use rand::{RngExt, distr::Alphanumeric};
use storage::{AppPaths, get_cache_path, get_config_path, get_data_path, get_temp_path};

use crate::profile::ProfileKind;

#[derive(Debug)]
pub struct ProfilePaths {
    profile_cache: Arc<PathBuf>,
    profile_config: Arc<PathBuf>,
    profile_data: Arc<PathBuf>,
    global_cache: Arc<PathBuf>,
    global_config: Arc<PathBuf>,
    global_data: Arc<PathBuf>,
    temp: Arc<PathBuf>,
    degraded: bool,
    is_temporary: bool,
}

impl ProfilePaths {
    pub fn new(kind: ProfileKind) -> Self {
        match kind {
            ProfileKind::Persistent { id } => {
                let id = id.unwrap_or_else(|| "default".to_string());
                let id = std::path::Path::new(&id)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .filter(|s| !s.is_empty())
                    .unwrap_or("default")
                    .to_string();
                let profile_name = Self::profile_name();

                let profile_cache = get_cache_path(vec![profile_name.clone()]).map(|p| p.join(&id));
                let profile_config = get_config_path(vec![profile_name.clone()]).map(|p| p.join(&id));
                let profile_data = get_data_path(vec![profile_name]).map(|p| p.join(&id));

                let global_cache = get_cache_path(vec![]).map(Arc::new);
                let global_config = get_config_path(vec![]).map(Arc::new);
                let global_data = get_data_path(vec![]).map(Arc::new);

                let degraded = profile_cache.is_none() || profile_config.is_none() || profile_data.is_none();

                let stable_temp = get_temp_path(None);
                let names = Self::dir_names();

                Self {
                    profile_cache: Arc::new(profile_cache.unwrap_or_else(|| stable_temp.join(&id).join(names.0))),
                    profile_config: Arc::new(profile_config.unwrap_or_else(|| stable_temp.join(&id).join(names.1))),
                    profile_data: Arc::new(profile_data.unwrap_or_else(|| stable_temp.join(&id).join(names.2))),
                    global_cache: global_cache.unwrap_or_else(|| stable_temp.join("global_cache").into()),
                    global_config: global_config.unwrap_or_else(|| stable_temp.join("global_config").into()),
                    global_data: global_data.unwrap_or_else(|| stable_temp.join("global_data").into()),
                    temp: Arc::new(stable_temp.join(&id)),
                    degraded,
                    is_temporary: false,
                }
            }
            ProfileKind::Temporary { custom_suffix } => {
                let suffix = custom_suffix.unwrap_or_else(|| {
                    rand::rng()
                        .sample_iter(Alphanumeric)
                        .take(6)
                        .map(char::from)
                        .collect()
                });

                let global_cache = get_cache_path(vec![]).map(Arc::new);
                let global_config = get_config_path(vec![]).map(Arc::new);
                let global_data = get_data_path(vec![]).map(Arc::new);

                let session_temp = get_temp_path(Some(&suffix));
                let names = Self::dir_names();

                Self {
                    profile_cache: Arc::new(session_temp.join(names.0)),
                    profile_config: Arc::new(session_temp.join(names.1)),
                    profile_data: Arc::new(session_temp.join(names.2)),
                    global_cache: global_cache.unwrap_or_else(|| session_temp.join("global_cache").into()),
                    global_config: global_config.unwrap_or_else(|| session_temp.join("global_config").into()),
                    global_data: global_data.unwrap_or_else(|| session_temp.join("global_data").into()),
                    temp: Arc::new(session_temp),
                    degraded: false,
                    is_temporary: true,
                }
            }
        }
    }

    pub fn profile_cache(&self) -> Arc<PathBuf> {
        Arc::clone(&self.profile_cache)
    }

    pub fn profile_config(&self) -> Arc<PathBuf> {
        Arc::clone(&self.profile_config)
    }

    pub fn profile_data(&self) -> Arc<PathBuf> {
        Arc::clone(&self.profile_data)
    }

    pub fn temp(&self) -> Arc<PathBuf> {
        Arc::clone(&self.temp)
    }

    pub fn is_degraded(&self) -> bool {
        self.degraded
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn profile_name() -> String {
        "Profiles".to_string()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn profile_name() -> String {
        "profiles".to_string()
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
            let _ = std::fs::remove_dir_all(&*self.profile_cache);
            let _ = std::fs::remove_dir_all(&*self.profile_config);
            let _ = std::fs::remove_dir_all(&*self.profile_data);
            let _ = std::fs::remove_dir_all(&*self.temp);
        }
    }
}

impl From<&ProfilePaths> for AppPaths {
    fn from(value: &ProfilePaths) -> Self {
        AppPaths {
            profile_cache: Arc::clone(&value.profile_cache),
            profile_config: Arc::clone(&value.profile_config),
            profile_data: Arc::clone(&value.profile_data),
            global_cache: Arc::clone(&value.global_cache),
            global_config: Arc::clone(&value.global_config),
            global_data: Arc::clone(&value.global_data),
            temp: Arc::clone(&value.temp),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_paths_persistent() {
        let profile_paths = ProfilePaths::new(ProfileKind::Persistent {
            id: Some("test_profile".to_string()),
        });
        let profiles_name = ProfilePaths::profile_name();

        assert!(!profile_paths.is_degraded());
        assert!(!profile_paths.is_temporary);
        assert_eq!(
            *profile_paths.profile_cache(),
            get_cache_path(vec![])
                .unwrap()
                .join(&profiles_name)
                .join("test_profile")
        );
        assert_eq!(
            *profile_paths.profile_config(),
            get_config_path(vec![])
                .unwrap()
                .join(&profiles_name)
                .join("test_profile")
        );
        assert_eq!(
            *profile_paths.profile_data(),
            get_data_path(vec![])
                .unwrap()
                .join(&profiles_name)
                .join("test_profile")
        );
        assert_eq!(*profile_paths.temp(), get_temp_path(None).join("test_profile"));
    }

    #[test]
    fn test_profile_paths_temporary() {
        let profile_paths = ProfilePaths::new(ProfileKind::Temporary {
            custom_suffix: Some("temp_suffix".to_string()),
        });
        assert!(!profile_paths.is_degraded());
        assert!(profile_paths.is_temporary);
        let temp_path = get_temp_path(Some("temp_suffix"));

        assert_eq!(*profile_paths.profile_cache(), temp_path.join(ProfilePaths::dir_names().0));
        assert_eq!(*profile_paths.profile_config(), temp_path.join(ProfilePaths::dir_names().1));
        assert_eq!(*profile_paths.profile_data(), temp_path.join(ProfilePaths::dir_names().2));
        assert_eq!(*profile_paths.temp(), temp_path);
    }
}
