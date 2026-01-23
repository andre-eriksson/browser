use std::path::PathBuf;

use constants::APP_NAME;

pub fn get_cache_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|p| p.join(APP_NAME))
}

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

pub fn create_paths(path: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}
