use std::path::PathBuf;

use constants::APP_NAME;

#[must_use]
pub fn get_cache_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|p| p.join(APP_NAME))
}

#[must_use]
pub fn get_config_path() -> Option<PathBuf> {
    let base_dir = dirs::config_dir()?.join(APP_NAME);

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        Some(base_dir.join("Config"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Some(base_dir)
    }
}

#[must_use]
pub fn get_data_path() -> Option<PathBuf> {
    let base_dir = dirs::data_dir()?.join(APP_NAME);

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        Some(base_dir.join("Data"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Some(base_dir)
    }
}

/// Create the necessary directories for the given path
///
/// # Errors
/// * If the directories cannot be created
pub fn create_paths(path: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}
