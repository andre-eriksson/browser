use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
};

use serde::{Serialize, de::DeserializeOwned};
use storage::paths::get_cache_path;

use crate::cache::{
    errors::CacheError,
    header::CacheHeader,
    index::{IndexFile, Pointer},
};

const LARGE_DIR: &str = "large";
const METADATA_FILE: &str = "metadata";
const CONTENT_FILE: &str = "content";

pub struct LargeFile;

impl LargeFile {
    pub fn write(data: &[u8], sha: [u8; 32], header: CacheHeader) -> Result<(), CacheError> {
        let str_sha = Self::hash_to_hex(&sha);
        let cache_path = match get_cache_path() {
            Some(path) => path,
            None => return Err(CacheError::WriteError(String::from("Cache path not found"))),
        };

        let path = cache_path
            .join(LARGE_DIR)
            .join(&str_sha[..2])
            .join(&str_sha[2..]);

        let _ = fs::create_dir_all(&path);

        let metadata_file = match OpenOptions::new()
            .append(true)
            .create(true)
            .open(path.join(METADATA_FILE))
        {
            Ok(file) => file,
            Err(e) => return Err(CacheError::IoError(e)),
        };

        let content_file = match OpenOptions::new()
            .append(true)
            .create(true)
            .open(path.join(CONTENT_FILE))
        {
            Ok(file) => file,
            Err(e) => return Err(CacheError::IoError(e)),
        };

        let mut idx_file = IndexFile::load().unwrap_or_default();
        idx_file.entries.insert(sha, Pointer::Large);
        idx_file.write()?;

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

        Ok(())
    }

    pub fn read<V>(sha: [u8; 32]) -> Option<(CacheHeader, V)>
    where
        V: Clone + Serialize + DeserializeOwned,
    {
        let str_sha = Self::hash_to_hex(&sha);

        let path = format!(
            "{}/{}/{}/{}",
            get_cache_path().unwrap().to_str().unwrap(),
            LARGE_DIR,
            &str_sha[..2],
            &str_sha[2..],
        );

        let metadata_path = format!("{}/{}", path, METADATA_FILE);
        let content_path = format!("{}/{}", path, CONTENT_FILE);

        if !std::path::Path::new(&metadata_path).exists()
            || !std::path::Path::new(&content_path).exists()
        {
            return None;
        }

        let meta_data = std::fs::read(&metadata_path).ok()?;
        let content_data = std::fs::read(&content_path).ok()?;

        let header: CacheHeader = postcard::from_bytes(&meta_data).ok()?;
        let content: V = postcard::from_bytes(&content_data).ok()?;

        Some((header, content))
    }

    fn hash_to_hex(hash: &[u8; 32]) -> String {
        hash.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
