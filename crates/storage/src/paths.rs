use std::{path::PathBuf, sync::Arc};

use manifest::APP_NAME;

#[derive(Debug, Clone)]
pub struct Directory {
    pub cache: Arc<PathBuf>,
    pub config: Arc<PathBuf>,
    pub data: Arc<PathBuf>,
    pub temp: Arc<PathBuf>,
}

impl Directory {
    pub fn try_new() -> Option<Self> {
        let cache = get_cache_path()?;
        let config = get_config_path()?;
        let data = get_data_path()?;
        let temp = get_temp_path(None);

        Some(Self {
            cache: Arc::new(cache),
            config: Arc::new(config),
            data: Arc::new(data),
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
pub fn get_cache_path() -> Option<PathBuf> {
    let base_dir = dirs::cache_dir()?;

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        Some(base_dir.join(APP_NAME).join("Cache"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Some(base_dir.join(APP_NAME.to_lowercase()))
    }
}

/// Get the path to the config directory for the application.
///
/// # Returns
/// * `Some(PathBuf)` - The path to the config directory if it can be determined
/// * `None` - If the config directory cannot be determined
#[must_use]
pub fn get_config_path() -> Option<PathBuf> {
    let base_dir = dirs::config_dir()?;

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        Some(base_dir.join(APP_NAME).join("Config"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Some(base_dir.join(APP_NAME.to_lowercase()))
    }
}

/// Get the path to the data directory for the application.
///
/// # Returns
/// * `Some(PathBuf)` - The path to the data directory if it can be determined
/// * `None` - If the data directory cannot be determined
#[must_use]
pub fn get_data_path() -> Option<PathBuf> {
    let base_dir = dirs::data_dir()?;

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        Some(base_dir.join(APP_NAME).join("Data"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Some(base_dir.join(APP_NAME.to_lowercase()))
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
