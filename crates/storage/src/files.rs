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

fn write_file<P, C>(path: P, content: C) -> std::io::Result<()>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)
}

fn read_file<P>(path: P) -> std::io::Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    std::fs::read(path)
}

pub fn read_file_from_cache<P>(path: P) -> std::io::Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    if let Some(mut cache_path) = get_cache_path() {
        cache_path.push(path);
        let path_str = cache_path.to_str().unwrap();
        read_file(path_str)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Cache directory not found",
        ))
    }
}

pub fn read_file_from_config<P>(path: P) -> std::io::Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    if let Some(mut config_path) = get_config_path() {
        config_path.push(path);
        let path_str = config_path.to_str().unwrap();
        read_file(path_str)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Config directory not found",
        ))
    }
}

pub fn read_file_from_data<P>(path: P) -> std::io::Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    if let Some(mut data_path) = get_data_path() {
        data_path.push(path);
        let path_str = data_path.to_str().unwrap();
        read_file(path_str)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Data directory not found",
        ))
    }
}

pub fn write_file_to_cache<P, C>(path: P, content: C) -> std::io::Result<()>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    if let Some(mut cache_path) = get_cache_path() {
        cache_path.push(path);
        let path_str = cache_path.to_str().unwrap();
        write_file(path_str, content)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Cache directory not found",
        ))
    }
}

pub fn write_file_to_config<P, C>(path: P, content: C) -> std::io::Result<()>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    if let Some(mut config_path) = get_config_path() {
        config_path.push(path);
        let path_str = config_path.to_str().unwrap();
        write_file(path_str, content)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Config directory not found",
        ))
    }
}

pub fn write_file_to_data<P, C>(path: P, content: C) -> std::io::Result<()>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    if let Some(mut data_path) = get_data_path() {
        data_path.push(path);
        let path_str = data_path.to_str().unwrap();
        write_file(path_str, content)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Data directory not found",
        ))
    }
}
