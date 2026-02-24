use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
};

use storage::paths::get_cache_path;

use crate::cache::{errors::CacheError, header::CacheHeader};

const LARGE_DIR: &str = "resources/large";
const METADATA_FILE: &str = "metadata";
const CONTENT_FILE: &str = "content";

pub struct LargeFile;

impl LargeFile {
    pub fn write(sha: [u8; 32], data: &[u8], header: &CacheHeader) -> Result<u32, CacheError> {
        let str_sha = Self::hash_to_hex(&sha);
        let cache_path = match get_cache_path() {
            Some(path) => path,
            None => return Err(CacheError::WriteError(String::from("Cache path not found"))),
        };

        let path = cache_path
            .join(LARGE_DIR)
            .join(&str_sha[..2])
            .join(&str_sha[2..]);

        fs::create_dir_all(&path)?;

        let metadata_file = match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path.join(METADATA_FILE))
        {
            Ok(file) => file,
            Err(e) => return Err(CacheError::IoError(e)),
        };

        let content_file = match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path.join(CONTENT_FILE))
        {
            Ok(file) => file,
            Err(e) => return Err(CacheError::IoError(e)),
        };

        let mut meta_writer = BufWriter::new(metadata_file);
        postcard::to_io(&header, &mut meta_writer)
            .map_err(|_| CacheError::WriteError(String::from("Failed to serialize header")))?;
        meta_writer
            .flush()
            .map_err(|_| CacheError::WriteError(String::from("Failed to flush header")))?;

        let mut content_writer = BufWriter::new(content_file);
        content_writer
            .write_all(data)
            .map_err(|_| CacheError::WriteError(String::from("Failed to write content data")))?;
        content_writer
            .flush()
            .map_err(|_| CacheError::WriteError(String::from("Failed to flush content data")))?;

        Ok(data.len() as u32)
    }

    pub fn read(sha: [u8; 32]) -> Result<(CacheHeader, Vec<u8>, usize), CacheError> {
        let str_sha = Self::hash_to_hex(&sha);

        let cache_path = match get_cache_path() {
            Some(path) => path,
            None => return Err(CacheError::WriteError(String::from("Cache path not found"))),
        };

        let path = cache_path
            .join(LARGE_DIR)
            .join(&str_sha[..2])
            .join(&str_sha[2..]);

        let metadata_path = path.join(METADATA_FILE);
        let content_path = path.join(CONTENT_FILE);

        if !std::path::Path::new(&metadata_path).exists()
            || !std::path::Path::new(&content_path).exists()
        {
            return Err(CacheError::ReadError(String::from("Large file not found")));
        }

        let meta_data = std::fs::read(&metadata_path)?;
        let content_data = std::fs::read(&content_path)?;
        let content_size = content_data.len();

        let header: CacheHeader =
            postcard::from_bytes(&meta_data).map_err(|_| CacheError::CorruptedHeader)?;

        Ok((header, content_data, content_size))
    }

    pub fn delete(sha: [u8; 32]) -> Result<(), CacheError> {
        let cache_path = get_cache_path()
            .ok_or_else(|| CacheError::WriteError(String::from("Cache path not found")))?;
        let str_sha = Self::hash_to_hex(&sha);

        let path = cache_path
            .join(LARGE_DIR)
            .join(&str_sha[..2])
            .join(&str_sha[2..]);

        if path.exists() {
            fs::remove_dir_all(&path)?;
        }

        Ok(())
    }

    fn hash_to_hex(hash: &[u8; 32]) -> String {
        hash.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
