use std::time::SystemTime;

use database::{Database, Table};
use rusqlite::Connection;

use crate::cache::{
    block::{BlockFile, MAX_BLOCK_SIZE},
    errors::CacheError,
    header::{CacheHeader, HEADER_VERSION},
    index::{Index, IndexDatabase, IndexEntry, IndexTable},
    large::LargeFile,
};

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

                        if header.url_hash != key
                            || header.content_size != content_size as u32
                            || header.header_version != HEADER_VERSION
                            || !header.is_fresh()
                        {
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
        if let Ok(connection) = IndexDatabase::open() {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let index = IndexTable::get_by_key(&connection, &key).map_err(|_| {
                CacheError::ReadError(String::from("Failed to read index entry from database"))
            })?;

            if let Some(idx) = index
                && idx.expires_at.unwrap_or(0) < now
            {
                Self::remove(key, idx.offset, idx.header_size, Some(&connection))?;
            }

            let mut entry = IndexEntry::Block;

            let (id, offset, header_size, content_size) = if value.len() > MAX_BLOCK_SIZE as usize {
                let values = LargeFile::write(value, key, &header)?;
                entry = IndexEntry::Large;

                (None, None, None, values)
            } else {
                let values = BlockFile::write(value, &mut header)?;

                (Some(values.0), Some(values.1), Some(values.2), values.3)
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

            let _ = IndexTable::insert(&connection, &index);
        }

        Ok(())
    }

    /// Removes a cache entry by its key, deleting both the index entry and the associated data file if it exists. If the entry does not exist, this function will simply return `Ok(())`.
    pub fn remove(
        key: [u8; 32],
        offset: Option<u32>,
        header_size: Option<u32>,
        connection: Option<&Connection>,
    ) -> Result<bool, CacheError> {
        match connection {
            Some(conn) => Self::delete(key, offset, header_size, conn),
            None => {
                if let Ok(conn) = IndexDatabase::open() {
                    let index = IndexTable::get_by_key(&conn, &key).map_err(|_| {
                        CacheError::ReadError(String::from(
                            "Failed to read index entry from database",
                        ))
                    })?;

                    let offset = index.as_ref().and_then(|idx| idx.offset);
                    let header_size = index.and_then(|idx| idx.header_size);

                    Self::delete(key, offset, header_size, &conn)
                } else {
                    Err(CacheError::WriteError(String::from(
                        "Failed to open index database",
                    )))
                }
            }
        }
    }

    /// Internal function to handle the deletion of cache entries, ensuring that both the index and the associated data files are removed.
    fn delete(
        key: [u8; 32],
        offset: Option<u32>,
        header_size: Option<u32>,
        connection: &Connection,
    ) -> Result<bool, CacheError> {
        let index = IndexTable::get_by_key(connection, &key).map_err(|_| {
            CacheError::ReadError(String::from("Failed to read index entry from database"))
        })?;

        if let Some(idx) = index {
            match idx.entry {
                IndexEntry::Large => LargeFile::delete(key)?,
                IndexEntry::Block => {
                    BlockFile::delete(idx.file_id, offset.unwrap_or(0), header_size.unwrap_or(0))?
                }
            }

            IndexTable::delete_by_key(connection, &key).map_err(|_| {
                CacheError::WriteError(String::from("Failed to delete index entry from database"))
            })?;

            return Ok(true);
        }

        Ok(false)
    }
}
