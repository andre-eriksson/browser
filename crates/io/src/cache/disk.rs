//! Disk-based cache implementation for storing and retrieving cached data on the filesystem, with support for both block-based and large file storage.

use std::time::SystemTime;

use database::{Database, Table};
use rusqlite::Connection;
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
        let connection = IndexDatabase::open()
            .map_err(|e| CacheError::ReadError(format!("Failed to open index database: {}", e)))?;

        let index = IndexTable::get_by_key(&connection, &key).map_err(|_| {
            CacheError::ReadError(String::from("Failed to read index entry from database"))
        })?;

        let idx = match index {
            Some(idx) => idx,
            None => return Ok(None),
        };

        let (header, value, content_size) = (match idx.entry {
            IndexEntry::Large => LargeFile::read(key),
            IndexEntry::Block => {
                let offset = idx.offset.ok_or(CacheError::CorruptedIndex)?;
                let header_size = idx.header_size.ok_or(CacheError::CorruptedIndex)?;

                BlockFile::read(idx.file_id, offset, header_size, idx.content_size)
            }
        })
        .map_err(|err| {
            if let CacheError::CorruptedIndex = err {
                let _ = Self::remove(key, Some(&connection));
            }

            err
        })?;

        let mut hasher = Sha256::new();
        hasher.update(&value);
        let content_hash: [u8; 32] = hasher.finalize().into();

        if header.url_hash != key
            || header.content_size != content_size as u32
            || header.header_version != HEADER_VERSION
            || !header.is_fresh()
            || header.content_hash != content_hash
            || header.dead
        {
            Self::remove(key, Some(&connection))?;
            return Ok(None);
        }

        Ok(Some(value))
    }

    /// Stores a value in the cache with the given key and header information.
    ///
    /// Data is written to disk **first** so that the actual offset is known before
    /// anything is recorded in the index.  The database therefore always contains
    /// the true offset — no speculative value that might need patching later.
    ///
    /// If an entry with the same key already exists it is removed before the new
    /// one is written.
    pub fn put(key: [u8; 32], value: &[u8], mut header: CacheHeader) -> Result<(), CacheError> {
        let connection = IndexDatabase::open()
            .map_err(|e| CacheError::WriteError(format!("Failed to open index database: {}", e)))?;

        connection
            .execute("BEGIN TRANSACTION", [])
            .map_err(|e| CacheError::WriteError(format!("Failed to begin transaction: {}", e)))?;

        let existing = IndexTable::get_by_key(&connection, &key).map_err(|_| {
            CacheError::ReadError(String::from("Failed to read index entry from database"))
        })?;

        if existing.is_some()
            && let Err(e) = Self::remove(key, Some(&connection))
        {
            connection.execute("ROLLBACK", []).ok();
            return Err(CacheError::WriteError(format!(
                "Failed to remove existing cache entry: {}",
                e
            )));
        }

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let (entry_type, file_id, offset, header_size, content_size) =
            if value.len() > MAX_BLOCK_SIZE as usize {
                LargeFile::write(key, value, &header)?;

                (IndexEntry::Large, 0u32, None, None, value.len() as u32)
            } else {
                let (fid, off, hsz, csz) = BlockFile::write(value, &mut header)?;

                (IndexEntry::Block, fid, Some(off), Some(hsz), csz)
            };

        let index = Index {
            key,
            entry: entry_type,
            file_id,
            offset,
            header_size,
            content_size,
            expires_at: header.expires_at,
            created_at: now,
            vary: header.vary.clone(),
        };

        if let Err(e) = IndexTable::insert(&connection, &index) {
            connection.execute("ROLLBACK", []).ok();

            match entry_type {
                IndexEntry::Large => {
                    LargeFile::delete(key).ok();
                }
                IndexEntry::Block => {
                    if let (Some(off), Some(hs)) = (offset, header_size) {
                        BlockFile::delete(file_id, off, hs).ok();
                    }
                }
            }

            return Err(CacheError::WriteError(format!(
                "Failed to insert index entry into database: {}",
                e
            )));
        }

        connection
            .execute("COMMIT", [])
            .map_err(|e| CacheError::WriteError(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    /// Removes a cache entry by its key, deleting both the index entry and the associated data file if it exists. If the entry does not exist, this function will simply return `Ok(())`.
    pub fn remove(key: [u8; 32], connection: Option<&Connection>) -> Result<bool, CacheError> {
        match connection {
            Some(conn) => Self::delete(key, conn),
            None => {
                let conn = IndexDatabase::open().map_err(|_| {
                    CacheError::WriteError(String::from("Failed to open index database"))
                })?;

                conn.execute("BEGIN TRANSACTION", []).map_err(|e| {
                    CacheError::WriteError(format!("Failed to begin transaction: {}", e))
                })?;

                let result = Self::delete(key, &conn);

                match result {
                    Ok(value) => {
                        conn.execute("COMMIT", []).map_err(|e| {
                            CacheError::WriteError(format!("Failed to commit transaction: {}", e))
                        })?;

                        Ok(value)
                    }
                    Err(e) => {
                        let _ = conn.execute("ROLLBACK", []);
                        Err(e)
                    }
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
