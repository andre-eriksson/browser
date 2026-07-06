use std::{path::PathBuf, sync::Arc};

use manifest::APP_NAME;

#[derive(Debug, Clone)]
pub struct Directory {
    pub profile_cache: Arc<PathBuf>,
    pub profile_config: Arc<PathBuf>,
    pub profile_data: Arc<PathBuf>,
    pub global_cache: Arc<PathBuf>,
    pub global_config: Arc<PathBuf>,
    pub global_data: Arc<PathBuf>,
    pub temp: Arc<PathBuf>,
}

impl Directory {
    pub fn try_new() -> Option<Self> {
        let cache = get_cache_path(vec![])?;
        let config = get_config_path(vec![])?;
        let data = get_data_path(vec![])?;
        let temp = get_temp_path(None);

        Some(Self {
            profile_cache: Arc::new(cache.clone()),
            profile_config: Arc::new(config.clone()),
            profile_data: Arc::new(data.clone()),
            global_cache: Arc::new(cache),
            global_config: Arc::new(config),
            global_data: Arc::new(data),
            temp: Arc::new(temp),
        })
    }
}

/// Get the path to the cache directory for the application.
///
/// # Returns
/// * `Some(PathBuf)` - The path to the cache directory if it can be determined
/// * `None` - If the cache directory cannot be determined
#[must_use]
pub fn get_cache_path(subdirs: Vec<String>) -> Option<PathBuf> {
    let base_dir = dirs::cache_dir()?;

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        let mut path = base_dir.join(APP_NAME);
        for subdir in subdirs {
            path.push(subdir);
        }

        Some(path.join("Cache"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let mut path = base_dir.join(APP_NAME.to_lowercase());
        for subdir in subdirs {
            path.push(subdir);
        }

        Some(path)
    }
}

/// Get the path to the config directory for the application.
///
/// # Returns
/// * `Some(PathBuf)` - The path to the config directory if it can be determined
/// * `None` - If the config directory cannot be determined
#[must_use]
pub fn get_config_path(subdirs: Vec<String>) -> Option<PathBuf> {
    let base_dir = dirs::config_dir()?;

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        let mut path = base_dir.join(APP_NAME);
        for subdir in subdirs {
            path.push(subdir);
        }

        Some(path.join("Config"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let mut path = base_dir.join(APP_NAME.to_lowercase());
        for subdir in subdirs {
            path.push(subdir);
        }

        Some(path)
    }
}

/// Get the path to the data directory for the application.
///
/// # Returns
/// * `Some(PathBuf)` - The path to the data directory if it can be determined
/// * `None` - If the data directory cannot be determined
#[must_use]
pub fn get_data_path(subdirs: Vec<String>) -> Option<PathBuf> {
    let base_dir = dirs::data_dir()?;

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        let mut path = base_dir.join(APP_NAME);
        for subdir in subdirs {
            path.push(subdir);
        }

        Some(path.join("Data"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let mut path = base_dir.join(APP_NAME.to_lowercase());
        for subdir in subdirs {
            path.push(subdir);
        }

        Some(path)
    }
}

/// Get the path to the temporary directory for the application.
///
/// # Returns
/// * `PathBuf` - The path to the temporary directory for the application
#[must_use]
pub fn get_temp_path(suffix: Option<&str>) -> PathBuf {
    let temp_dir = std::env::temp_dir();

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        match suffix {
            Some(s) => temp_dir.join(format!("{}-{}", APP_NAME, s)),
            None => temp_dir.join(APP_NAME),
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        match suffix {
            Some(s) => temp_dir.join(format!("{}-{}", APP_NAME.to_lowercase(), s)),
            None => temp_dir.join(APP_NAME.to_lowercase()),
        }
    }
}

/// Create the necessary directories for the given path
///
/// # Errors
/// * If the directories cannot be created
pub fn create_paths(path: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}
