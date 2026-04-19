use std::path::PathBuf;

use manifest::APP_NAME;

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
pub fn get_temp_path() -> PathBuf {
    let temp_dir = std::env::temp_dir();

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        temp_dir.join(APP_NAME)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        temp_dir.join(APP_NAME.to_lowercase())
    }
}

/// Create the necessary directories for the given path
///
/// # Errors
/// * If the directories cannot be created
pub fn create_paths(path: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}
