use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    time::SystemTime,
};

use database::{Database, Table};
use serde::{Serialize, de::DeserializeOwned};
use storage::paths::get_cache_path;

use crate::cache::{
    errors::CacheError,
    header::CacheHeader,
    index::{Index, IndexDatabase, IndexEntry, IndexTable},
};

const LARGE_DIR: &str = "resources/large";
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
            .write(true)
            .create(true)
            .truncate(true)
            .open(path.join(CONTENT_FILE))
        {
            Ok(file) => file,
            Err(e) => return Err(CacheError::IoError(e)),
        };

        if let Ok(connection) = IndexDatabase::open() {
            let index = Index {
                key: sha,
                entry: IndexEntry::Large,
                file_id: 0,
                offset: None,
                header_size: None,
                content_size: data.len() as u32,
                expires_at: None,
                created_at: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                vary: header.vary.clone(),
            };

            let _ = IndexTable::insert(&connection, &index);
        }

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

    pub fn read<V>(sha: [u8; 32]) -> Result<(CacheHeader, V, usize), CacheError>
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
            return Err(CacheError::ReadError(String::from("Large file not found")));
        }

        let meta_data = std::fs::read(&metadata_path)?;
        let content_data = std::fs::read(&content_path)?;

        let header: CacheHeader =
            postcard::from_bytes(&meta_data).map_err(|_| CacheError::CorruptedHeader)?;
        let content: V = postcard::from_bytes(&content_data).map_err(|_| {
            CacheError::ReadError(String::from("Failed to deserialize content data"))
        })?;

        Ok((header, content, content_data.len()))
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
