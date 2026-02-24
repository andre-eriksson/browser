//! Large file handling for the cache system.

use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
};

use storage::paths::get_cache_path;

use crate::cache::{errors::CacheError, header::CacheHeader};

/// The main directory for storing large cache entries.
const LARGE_DIR: &str = "resources/large";

/// Metadata file name for storing the cache header information of a large entry.
const METADATA_FILE: &str = "metadata";

/// Content file name for storing the actual cached data of a large entry.
const CONTENT_FILE: &str = "content";

/// The interface for handling large files in the cache system, providing methods to write, read, and delete large cache
/// entries on the filesystem. Each large entry is stored in a separate directory based on its SHA-256 hash, with
/// metadata and content stored in separate files for efficient access and management.
pub struct LargeFile;

impl LargeFile {
    /// Writes a large cache entry to the filesystem, creating a directory structure based on the SHA-256 hash of the
    /// entry's URL. It stores the cache header metadata in a separate file and the actual content in another file.
    /// The method returns the size of the content written or an error if the operation fails.
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

    /// Reads a large cache entry from the filesystem based on its SHA-256 hash. It retrieves both the cache header metadata
    /// and the actual content data, returning them along with the size of the content. If the entry is not found or if
    /// there are any issues during reading, it returns an appropriate error.
    pub fn read(sha: [u8; 32]) -> Result<(CacheHeader, Vec<u8>, usize), CacheError> {
        let str_sha = Self::hash_to_hex(&sha);

        let cache_path = match get_cache_path() {
            Some(path) => path,
            None => return Err(CacheError::ReadError(String::from("Cache path not found"))),
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

    /// Deletes a large cache entry from the filesystem based on its SHA-256 hash. It removes the entire directory associated
    /// with the entry, including both the metadata and content files. If the entry does not exist, it simply returns `Ok(())`,
    /// ensuring that the method is idempotent and does not fail if the entry is already absent.
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

    /// Converts a SHA-256 hash byte array into a hexadecimal string representation, which is used for naming directories
    /// in the filesystem. This method iterates over each byte in the hash and formats it as a two-digit hexadecimal string,
    /// concatenating them to produce the final string representation of the hash.
    fn hash_to_hex(hash: &[u8; 32]) -> String {
        hash.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
