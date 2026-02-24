//! Disk-based cache implementation for storing and retrieving cached data on the filesystem, with support for both block-based and large file storage.

use std::time::SystemTime;

use database::{Database, Table};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};

use crate::cache::{
    block::{BlockFile, MAX_BLOCK_SIZE},
    errors::CacheError,
    header::{CacheHeader, HEADER_VERSION},
    index::{Index, IndexDatabase, IndexEntry, IndexTable},
    large::LargeFile,
};

/// Main interface for the disk cache, providing methods to get, put, and remove cached entries.
/// It handles both block-based and large file storage, ensuring data integrity and proper cleanup
/// of expired or corrupted entries.
pub struct DiskCache;

impl DiskCache {
    /// Retrieves a cached value by its key, returning `None` if the entry is not found or is expired.
    pub fn get(key: [u8; 32]) -> Result<Option<Vec<u8>>, CacheError> {
        match IndexDatabase::open() {
            Ok(connection) => {
                let index = IndexTable::get_by_key(&connection, &key).map_err(|_| {
                    CacheError::ReadError(String::from("Failed to read index entry from database"))
                })?;

                match index {
                    Some(idx) => {
                        let (header, value, content_size) = match idx.entry {
                            IndexEntry::Large => LargeFile::read(key),
                            IndexEntry::Block => BlockFile::read(
                                idx.file_id,
                                idx.offset.unwrap_or(0),
                                idx.header_size.unwrap_or(0),
                                idx.content_size,
                            ),
                        }?;

                        let mut hasher = Sha256::new();
                        hasher.update(&value);
                        let content_hash: [u8; 32] = hasher.finalize().into();

                        if header.url_hash != key
                            || header.content_size != content_size as u32
                            || header.header_version != HEADER_VERSION
                            || !header.is_fresh()
                            || header.content_hash != content_hash
                        {
                            Self::remove(key, Some(&connection))?;
                            return Ok(None);
                        }

                        Ok(Some(value))
                    }
                    None => Ok(None),
                }
            }
            Err(e) => Err(CacheError::ReadError(format!(
                "Failed to open index database: {}",
                e
            ))),
        }
    }

    /// Stores a value in the cache with the given key and header information. If an entry with the same key already exists and is expired, it will be overwritten.
    pub fn put(key: [u8; 32], value: &[u8], mut header: CacheHeader) -> Result<(), CacheError> {
        match IndexDatabase::open() {
            Ok(connection) => {
                connection.execute("BEGIN TRANSACTION", []).map_err(|e| {
                    CacheError::WriteError(format!("Failed to begin transaction: {}", e))
                })?;

                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let index = IndexTable::get_by_key(&connection, &key).map_err(|_| {
                    CacheError::ReadError(String::from("Failed to read index entry from database"))
                })?;

                if index.is_some() {
                    let removal_result = Self::remove(key, Some(&connection));

                    if removal_result.is_err() {
                        connection.execute("ROLLBACK", []).ok();
                        return Err(CacheError::WriteError(format!(
                            "Failed to remove existing cache entry: {}",
                            removal_result.err().unwrap()
                        )));
                    }
                }

                let mut entry = IndexEntry::Block;

                let (id, offset, header_size, content_size, path) =
                    if value.len() > MAX_BLOCK_SIZE as usize {
                        entry = IndexEntry::Large;

                        (None, None, None, value.len() as u32, None)
                    } else {
                        let values = BlockFile::prepare_write(value, &mut header)?;

                        (
                            Some(values.0),
                            Some(values.1),
                            Some(values.2),
                            values.3,
                            Some(values.4),
                        )
                    };

                let index = Index {
                    key,
                    entry,
                    file_id: id.unwrap_or(0),
                    offset,
                    header_size,
                    content_size,
                    expires_at: header.expires_at,
                    created_at: now,
                    vary: header.vary.clone(),
                };

                let insert_result = IndexTable::insert(&connection, &index).map_err(|_| {
                    CacheError::WriteError(String::from("Failed to write index entry to database"))
                });

                match insert_result {
                    Ok(_) => {
                        connection.execute("COMMIT", []).map_err(|e| {
                            CacheError::WriteError(format!("Failed to commit transaction: {}", e))
                        })?;

                        match index.entry {
                            IndexEntry::Large => {
                                LargeFile::write(key, value, &header)?;
                            }
                            IndexEntry::Block => {
                                if let (Some(fid), Some(path)) = (id, path) {
                                    BlockFile::write(value, fid, &path, &header)?;
                                }

                                if let Some(off) = offset {
                                    connection
                                        .execute(
                                            "UPDATE cache_index SET offset = ?1 WHERE key = ?2",
                                            params![off, key],
                                        )
                                        .ok();
                                }
                            }
                        }

                        Ok(())
                    }
                    Err(e) => {
                        connection.execute("ROLLBACK", []).ok();

                        match &index.entry {
                            IndexEntry::Large => {
                                LargeFile::delete(key).ok();
                            }
                            IndexEntry::Block => {
                                if let (Some(fid), Some(off), Some(hs)) = (id, offset, header_size)
                                {
                                    BlockFile::delete(fid, off, hs).ok();
                                }
                            }
                        }

                        Err(CacheError::WriteError(format!(
                            "Failed to insert index entry into database: {}",
                            e
                        )))
                    }
                }
            }
            Err(e) => Err(CacheError::WriteError(format!(
                "Failed to open index database: {}",
                e
            ))),
        }
    }

    /// Removes a cache entry by its key, deleting both the index entry and the associated data file if it exists. If the entry does not exist, this function will simply return `Ok(())`.
    pub fn remove(key: [u8; 32], connection: Option<&Connection>) -> Result<bool, CacheError> {
        match connection {
            Some(conn) => Self::delete(key, conn),
            None => {
                if let Ok(conn) = IndexDatabase::open() {
                    conn.execute("BEGIN TRANSACTION", []).map_err(|e| {
                        CacheError::WriteError(format!("Failed to begin transaction: {}", e))
                    })?;

                    let result = Self::delete(key, &conn);

                    conn.execute("COMMIT", []).map_err(|e| {
                        CacheError::WriteError(format!("Failed to commit transaction: {}", e))
                    })?;

                    result
                } else {
                    Err(CacheError::WriteError(String::from(
                        "Failed to open index database",
                    )))
                }
            }
        }
    }

    /// Internal function to handle the deletion of cache entries, ensuring that both the index and the associated data files are removed.
    fn delete(key: [u8; 32], connection: &Connection) -> Result<bool, CacheError> {
        let index = IndexTable::get_by_key(connection, &key).map_err(|_| {
            CacheError::ReadError(String::from("Failed to read index entry from database"))
        })?;

        if let Some(idx) = index {
            match idx.entry {
                IndexEntry::Large => LargeFile::delete(key)?,
                IndexEntry::Block => BlockFile::delete(
                    idx.file_id,
                    idx.offset.unwrap_or(0),
                    idx.header_size.unwrap_or(0),
                )?,
            }

            IndexTable::delete_by_key(connection, &key).map_err(|_| {
                CacheError::WriteError(String::from("Failed to delete index entry from database"))
            })?;

            return Ok(true);
        }

        Ok(false)
    }
}
