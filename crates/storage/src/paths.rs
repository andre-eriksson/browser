use std::path::PathBuf;

use constants::APP_NAME;

pub fn get_cache_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|p| p.join(APP_NAME))
}

pub fn get_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join(APP_NAME))
}

pub fn get_data_path() -> Option<PathBuf> {
    dirs::data_dir().map(|p| p.join(APP_NAME))
}

pub fn create_paths(path: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}
