//! Disk-based cache implementation for storing and retrieving cached data on the filesystem, with support for both block-based and large file storage.

use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    time::SystemTime,
};

use database::Table;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use storage::get_cache_path;

use crate::cache::{
    block::{BLOCK_DIR, BlockFile, BlockHeader, MAGIC, MAX_BLOCK_SIZE, VERSION},
    errors::CacheError,
    header::{CacheHeader, HEADER_VERSION},
    index::{Index, IndexDatabase, IndexEntry, IndexTable},
    large::LargeFile,
};

/// Minimum block file size before compaction is considered (16MB).
const COMPACTION_THRESHOLD: u64 = 16 * 1024 * 1024;
/// Minimum dead-byte ratio within entries to trigger compaction (50%).
const COMPACTION_DEAD_THRESHOLD: usize = 2;

struct ScannedEntry {
    url_hash: [u8; 32],
    header_bytes: Vec<u8>,
    content_bytes: Vec<u8>,
    is_dead: bool,
}

/// Main interface for the disk cache, providing methods to get, put, and remove cached entries.
/// It handles both block-based and large file storage, ensuring data integrity and proper cleanup
/// of expired or corrupted entries.
#[derive(Debug, Clone)]
pub struct DiskCache {
    database: IndexDatabase,
}

impl DiskCache {
    pub fn new(database: IndexDatabase) -> Self {
        Self { database }
    }

    /// Retrieves a cached value by its key, returning `None` if the entry is not found or is expired.
    pub fn get(&self, key: [u8; 32]) -> Result<Option<Vec<u8>>, CacheError> {
        let Ok(connection) = self.database.connection.lock() else {
            return Err(CacheError::DatabaseLock);
        };

        self.get_with_connection(key, &connection)
    }

    /// Retrieves a cached value by its key, using the provided connection, returning `None` if the entry is not
    /// found or is expired.
    fn get_with_connection(&self, key: [u8; 32], connection: &Connection) -> Result<Option<Vec<u8>>, CacheError> {
        let index = IndexTable::get_by_key(connection, &key).map_err(CacheError::Database)?;

        let Some(idx) = index else { return Ok(None) };

        let (header, value, content_size) = (match idx.entry {
            IndexEntry::Large => LargeFile::read(key),
            IndexEntry::Block => {
                let offset = idx.offset.ok_or(CacheError::CorruptedIndex)?;
                let header_size = idx.header_size.ok_or(CacheError::CorruptedIndex)?;

                BlockFile::read(idx.file_id, offset, header_size, idx.content_size)
            }
        })?;

        let mut hasher = Sha256::new();
        hasher.update(&value);
        let content_hash: [u8; 32] = hasher.finalize().into();

        if header.url_hash != key
            || header.content_size != u32::try_from(content_size).unwrap_or(u32::MAX)
            || header.header_version != HEADER_VERSION
            || !header.is_fresh()
            || header.content_hash != content_hash
            || header.dead
        {
            // self.remove(key, Some(&connection))?;
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
    /// If an entry with the same key already exists and has identical content (same content hash),
    /// the write is skipped to avoid duplicate storage. Otherwise, the existing entry is removed
    /// before the new one is written.
    pub fn put(&self, key: [u8; 32], value: &[u8], mut header: CacheHeader) -> Result<(), CacheError> {
        let Ok(connection) = self.database.connection.lock() else {
            return Err(CacheError::DatabaseLock);
        };

        connection
            .execute("BEGIN TRANSACTION", [])
            .map_err(CacheError::Database)?;

        let existing = IndexTable::get_by_key(&connection, &key).map_err(CacheError::Database)?;

        if existing.is_some() {
            if let Ok(Some(existing_value)) = self.get_with_connection(key, &connection) {
                let mut hasher = Sha256::new();
                hasher.update(&existing_value);
                let existing_content_hash: [u8; 32] = hasher.finalize().into();

                let mut new_hasher = Sha256::new();
                new_hasher.update(value);
                let new_content_hash: [u8; 32] = new_hasher.finalize().into();

                if existing_content_hash == new_content_hash {
                    connection.execute("COMMIT", []).ok();
                    return Ok(());
                }
            }

            if let Err(e) = Self::delete_with_connection(key, &connection) {
                connection.execute("ROLLBACK", []).ok();
                return Err(CacheError::Write(format!("failed to remove existing cache entry: {e}")));
            }
        }

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let (entry_type, file_id, offset, header_size, content_size) =
            if value.len() > usize::try_from(MAX_BLOCK_SIZE).unwrap_or(usize::MAX) {
                LargeFile::write(key, value, &header)?;

                (IndexEntry::Large, 0u32, None, None, u32::try_from(value.len()).unwrap_or(u32::MAX))
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
            expires_at: header.expires_at.map(|v| v as i64),
            created_at: isize::try_from(now).unwrap_or(isize::MAX),
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

            return Err(CacheError::Database(e));
        }

        connection
            .execute("COMMIT", [])
            .map_err(CacheError::Database)?;

        Ok(())
    }

    /// Removes a cache entry by its key, deleting both the index entry and the associated data file if it exists.
    /// If the entry does not exist, this function will simply return `Ok(())`.
    pub fn delete(&mut self, key: [u8; 32]) -> Result<bool, CacheError> {
        let Ok(connection) = self.database.connection.lock() else {
            return Err(CacheError::DatabaseLock);
        };

        Self::delete_with_connection(key, &connection)
    }

    /// Removes a cache entry by its key, using the provided connection, deleting both the index entry and the a
    /// ssociated data file if it exists. If the entry does not exist, this function will simply return `Ok(())`.
    fn delete_with_connection(key: [u8; 32], connection: &Connection) -> Result<bool, CacheError> {
        let index = IndexTable::get_by_key(connection, &key).map_err(CacheError::Database)?;

        if let Some(idx) = index {
            match idx.entry {
                IndexEntry::Large => LargeFile::delete(key)?,
                IndexEntry::Block => {
                    let offset = idx.offset.ok_or(CacheError::CorruptedIndex)?;
                    let header_size = idx.header_size.ok_or(CacheError::CorruptedIndex)?;

                    BlockFile::delete(idx.file_id, offset, header_size)?;
                }
            }

            IndexTable::delete_by_key(connection, &key).map_err(CacheError::Database)?;

            return Ok(true);
        }

        Ok(false)
    }

    /// Compacts block files on a per-file basis.
    ///
    /// For each `.bin` file in the blocks directory that is nearly full (>= 80% of
    /// `MAX_BLOCK_SIZE`) and has a high proportion of dead entries (>= 50% of entry bytes),
    /// this rewrites the file with only the live entries, reclaiming space.
    ///
    /// After compaction the index database is updated with the new offsets so that
    /// subsequent reads resolve correctly. Dead entries are also pruned from the index.
    #[allow(dead_code)]
    // NOTE: Will be used when a scheduler is implemented to run compaction in
    //       the background every N hours or when certain thresholds are met.
    pub fn compact(&mut self) -> Result<(), CacheError> {
        let Some(cache_path) = get_cache_path() else {
            return Err(CacheError::CacheDirectoryNotFound);
        };

        let block_dir = cache_path.join(BLOCK_DIR);
        if !block_dir.exists() {
            return Ok(());
        }

        let mut files: Vec<(PathBuf, u32)> = fs::read_dir(&block_dir)?
            .flatten()
            .filter(|e| e.path().is_file() && e.path().extension().is_some_and(|ext| ext == "bin"))
            .filter_map(|e| {
                let path = e.path();
                let num: u32 = path.file_stem()?.to_str()?.parse().ok()?;
                Some((path, num))
            })
            .collect();

        files.sort_by_key(|(_, num)| *num);

        for (path, _file_number) in &files {
            let metadata = fs::metadata(path)?;
            let file_size = metadata.len();

            if file_size < COMPACTION_THRESHOLD {
                continue;
            }

            let data = fs::read(path)?;

            let (block_header, remaining): (BlockHeader, &[u8]) =
                postcard::take_from_bytes(&data).map_err(|_| CacheError::CorruptedBlock)?;

            if block_header.magic != MAGIC || block_header.version != VERSION {
                continue;
            }

            let mut entries: Vec<ScannedEntry> = Vec::new();
            let mut total_entry_bytes = 0;
            let mut dead_entry_bytes = 0;
            let mut cursor: &[u8] = remaining;

            while !cursor.is_empty() {
                let (header, after_header): (CacheHeader, &[u8]) = match postcard::take_from_bytes(cursor) {
                    Ok(result) => result,
                    Err(_) => break,
                };

                let header_size = cursor.len() - after_header.len();
                let content_size = header.content_size as usize;

                if after_header.len() < content_size {
                    break;
                }

                let entry_size = header_size + content_size;
                total_entry_bytes += entry_size;

                if header.dead {
                    dead_entry_bytes += entry_size;
                }

                entries.push(ScannedEntry {
                    url_hash: header.url_hash,
                    header_bytes: cursor[..header_size].to_vec(),
                    content_bytes: after_header[..content_size].to_vec(),
                    is_dead: header.dead,
                });

                cursor = &after_header[content_size..];
            }

            if total_entry_bytes == 0 {
                continue;
            }

            if total_entry_bytes.div_ceil(dead_entry_bytes) < COMPACTION_DEAD_THRESHOLD {
                continue;
            }

            let temp_path = path.with_extension("tmp");

            let mut block_header_size =
                u32::try_from(data.len() - remaining.len()).map_err(|_| CacheError::CorruptedBlock)?;

            let mut new_file = File::create(&temp_path).map_err(CacheError::Io)?;

            new_file.write_all(&data[..block_header_size as usize])?;

            for entry in &entries {
                let Ok(conn) = self.database.connection.lock() else {
                    continue;
                };

                if entry.is_dead {
                    let _ = IndexTable::delete_by_key(&conn, &entry.url_hash);
                    continue;
                }

                new_file.write_all(&entry.header_bytes)?;
                new_file.write_all(&entry.content_bytes)?;

                let header_size = u32::try_from(entry.header_bytes.len()).unwrap_or(u32::MAX);

                let _ = IndexTable::update_block_offset(&conn, &entry.url_hash, block_header_size, header_size);

                block_header_size += header_size + u32::try_from(entry.content_bytes.len()).unwrap_or(u32::MAX);
            }

            new_file.flush()?;

            fs::rename(&temp_path, path)?;
        }

        Ok(())
    }
}
