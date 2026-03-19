use std::path::Path;

use crate::paths::{get_cache_path, get_config_path, get_data_path};

#[macro_export]
macro_rules! write_tmp {
    ($name:expr, $content:expr) => {{
        use std::fs;
        use std::path::PathBuf;

        let mut path = PathBuf::from("tmp");

        let _ = fs::create_dir_all(&path);
        path.push($name);
        let _ = fs::write(&path, $content);
    }};
}

fn read_file<P>(path: P) -> std::io::Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    std::fs::read(path)
}

/// Read a file from the cache directory
///
/// # Arguments
/// * `path` - The path to the file relative to the cache directory
///
/// # Errors
/// * If the cache directory is not found
/// * If the file cannot be read
pub fn read_file_from_cache<P>(path: P) -> std::io::Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    if let Some(mut cache_path) = get_cache_path() {
        cache_path.push(path);

        let Some(path_str) = cache_path.to_str() else {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid cache path"));
        };

        read_file(path_str)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Cache directory not found"))
    }
}

/// Read a file from the config directory
///
/// # Arguments
/// * `path` - The path to the file relative to the config directory
///
/// # Errors
/// * If the config directory is not found
/// * If the file cannot be read
pub fn read_file_from_config<P>(path: P) -> std::io::Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    if let Some(mut config_path) = get_config_path() {
        config_path.push(path);

        let Some(path_str) = config_path.to_str() else {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid config path"));
        };

        read_file(path_str)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Config directory not found"))
    }
}

/// Read a file from the data directory
///
/// # Arguments
/// * `path` - The path to the file relative to the data directory
///
/// # Errors
/// * If the data directory is not found
/// * If the file cannot be read
pub fn read_file_from_data<P>(path: P) -> std::io::Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    if let Some(mut data_path) = get_data_path() {
        data_path.push(path);

        let Some(path_str) = data_path.to_str() else {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data path"));
        };

        read_file(path_str)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Data directory not found"))
    }
}
