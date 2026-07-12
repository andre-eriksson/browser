//! This module defines the `Index` struct and related database operations for managing cache entries in
//! the browser's caching system. It provides functionality to store metadata about cached resources,
//! including their location in block files or large files, expiration times, and other relevant information.
//! The `IndexDatabase` struct implements the `Database` trait to handle database connections and schema
//! management, while the `IndexTable` struct implements the `Table` trait to manage CRUD operations on
//! the index entries.

use std::{
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use database::{Database, Table};
use http::{
    HeaderMap, HeaderValue,
    header::{IF_MODIFIED_SINCE, IF_NONE_MATCH},
};
use httpdate::fmt_http_date;
use rusqlite::{Connection, Result, params};
use storage::Directory;

use crate::http::HttpCache;

/// Path to the `SQLite` database file that stores the cache index entries.
#[cfg(not(test))]
const IDX_DATABASE: &str = "resources/index.db";
#[cfg(test)]
const IDX_DATABASE: &str = "tests/resources/index.db";

/// Enum representing the type of cache entry, indicating whether the cached resource is stored in a block file or as a large file.
#[derive(Debug, Clone, Copy)]
pub enum IndexEntry {
    /// The cached resource is stored in a block file, which may contain multiple entries and requires offset and header size for access.
    Block,

    /// The cached resource is stored as a large file, which is accessed directly without the need for offsets or header sizes.
    Large,
}

/// Represents a cache index entry, containing metadata about a cached resource and its location in the cache storage.
#[derive(Debug)]
pub struct Index {
    // Identity
    /// The SHA-256 hash of the cached resource's URL, used as the key for lookup in the index.
    pub key: [u8; 32],

    // Location
    /// Indicates whether the entry is stored in a block file or as a large file, which determines how the content is accessed.
    pub entry: IndexEntry,

    /// The ID of the block file where the content is stored (for block entries) or a reference for large files.
    pub file_id: u32,

    /// The byte offset within the block file where the content starts (only applicable for block entries).
    pub offset: Option<u32>,

    /// The size of the header metadata for the cached entry, used to correctly read the content from block files.
    pub header_size: Option<u32>,

    /// The size of the cached content in bytes, used for validation and to determine how much data to read.
    pub content_size: u32,

    // HTTP cache metadata
    /// Optional `ETag` value from the HTTP response, used for conditional requests during revalidation.
    pub etag: Option<String>,

    /// The UNIX timestamp (in seconds) when the cached entry expires, used to determine if the entry is still valid.
    pub expires_at: Option<i64>,

    /// When the resources was last fetched/updated.
    pub fetched_at: i64,

    /// Optional last modified time of the cached content, represented as a UNIX timestamp in seconds.
    pub last_modified: Option<i64>,

    /// Optional max-age directive from the `Cache-Control` header, representing the maximum age of the cached content in seconds.
    pub max_age_seconds: Option<i64>,

    /// The header names associated with the cached entry that affect its cache key, derived from the Vary header, used for cache validation.
    pub vary: Vec<String>,

    /// The SHA-256 hash of the values of the headers specified in the Vary header, used to differentiate cache entries based on varying request headers.
    pub vary_hash: [u8; 32],

    // Revalidation state
    /// Indicates whether the cached entry must be revalidated with the origin server before being served,
    /// based on the `must-revalidate` directive in the `Cache-Control` header.
    pub must_revalidate: bool,

    /// Indicates whether the cached entry should not be served without revalidation, based on the `no-cache`
    /// directive in the `Cache-Control` header.
    pub no_cache: bool,

    // Metadata
    /// The UNIX timestamp (in seconds) when the cached entry was created, used for cache management and eviction policies.
    pub created_at: isize,
}

impl Index {
    /// Determines if the cached entry is still fresh based on its metadata and the current time.
    /// This method checks the following conditions in order:
    /// 1. If the `no-cache` directive is present, the entry is not fresh and must be revalidated.
    /// 2. If the `expires_at` timestamp is set, the entry is fresh if the current time is before
    ///    the expiration time.
    /// 3. If the `max_age_seconds` directive is set, the entry is fresh if the current time is within
    ///    the max-age window from the `fetched_at` time.
    /// 4. If the `last_modified` timestamp is set, a heuristic freshness lifetime is calculated as
    ///    one-tenth of the time since the last modification, and the entry is fresh if the current
    ///    time is within this heuristic window from the `fetched_at` time.
    /// 5. If none of the above conditions apply, the entry is considered fresh by default.
    pub fn is_fresh(&self) -> bool {
        if self.no_cache {
            return false;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .min(i64::MAX as u64) as i64;

        if let Some(expires_at) = self.expires_at {
            return now < expires_at;
        }

        if let Some(max_age) = self.max_age_seconds {
            return now < self.fetched_at + max_age;
        }

        if let Some(last_modified) = self.last_modified {
            let heuristic_ttl = (self.fetched_at.saturating_sub(last_modified)) / 10;
            return now < self.fetched_at + heuristic_ttl;
        }

        true
    }

    /// Generates the necessary headers for revalidating the cached entry with the origin server. This includes:
    /// - `If-None-Match` header with the `ETag` value if it is present, allowing the server to respond with a
    ///   304 Not Modified if the content has not changed.
    /// - `If-Modified-Since` header with the last modified time if it is present, allowing the server to
    ///   respond with a 304 Not Modified if the content has not changed since that time.
    pub fn revalidation_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::with_capacity(3);

        if let Some(ref etag) = self.etag
            && let Ok(value) = HeaderValue::from_bytes(etag.as_bytes())
        {
            headers.insert(IF_NONE_MATCH, value);
        }

        if let Some(lm) = self.last_modified
            && let Ok(secs) = u64::try_from(lm)
        {
            let last_modified_time = UNIX_EPOCH + Duration::from_secs(secs);
            let last_modified_str = fmt_http_date(last_modified_time);

            if let Ok(value) = HeaderValue::from_bytes(last_modified_str.as_bytes()) {
                headers.insert(IF_MODIFIED_SINCE, value);
            }
        }

        headers
    }
}

/// Database interface for managing cache index entries, providing methods to open the database connection and ensure the schema is set up correctly.
#[derive(Debug, Clone)]
pub struct IndexDatabase {
    pub connection: Arc<Mutex<Connection>>,
}

impl Database for IndexDatabase {
    fn open(dirs: Directory) -> Result<Self> {
        let path = dirs.profile_cache.join(IDX_DATABASE);

        std::fs::create_dir_all(path.parent().unwrap())
            .map_err(|_| rusqlite::Error::InvalidPath("Failed to create cache directory".into()))?;

        let conn = Connection::open(path)?;

        conn.execute_batch("PRAGMA journal_mode = WAL;")?;

        IndexTable::create_table(&conn)?;

        Ok(Self {
            connection: Arc::new(Mutex::new(conn)),
        })
    }
}

/// Table interface for managing cache index entries, providing methods to create the index table and perform CRUD operations on the entries.
pub struct IndexTable;

impl IndexTable {
    /// Retrieves an index entry by its key, ensuring it is not expired.
    pub fn get_by_key(conn: &Connection, key: &[u8; 32]) -> Result<Vec<Index>> {
        let mut stmt = conn.prepare(
            "SELECT
                key,
                entry_type,
                file_id,
                offset,
                header_size,
                content_size,
                etag,
                expires_at,
                fetched_at,
                last_modified,
                max_age_seconds,
                vary,
                vary_hash,
                must_revalidate,
                no_cache,
                created_at
            FROM cache_index WHERE key = ?1",
        )?;
        let mut rows = stmt.query(params![key])?;

        let mut entries = Vec::new();

        while let Some(row) = rows.next()? {
            let entry_type: String = row.get(1)?;
            let entry = match entry_type.as_str() {
                "block" => IndexEntry::Block,
                "large" => IndexEntry::Large,
                _ => return Ok(vec![]),
            };

            let expires_at = row.get::<usize, Option<i64>>(7)?;
            let vary = HttpCache::deserialize_vary(row.get::<usize, Option<String>>(11)?);

            entries.push(Index {
                key: row.get(0)?,
                entry,
                file_id: row.get(2)?,
                offset: row.get(3)?,
                header_size: row.get(4)?,
                content_size: row.get(5)?,
                etag: row.get(6)?,
                expires_at,
                fetched_at: row.get(8)?,
                last_modified: row.get(9)?,
                max_age_seconds: row.get(10)?,
                vary,
                vary_hash: row.get(12)?,
                must_revalidate: row.get(13)?,
                no_cache: row.get(14)?,
                created_at: row.get(15)?,
            })
        }

        Ok(entries)
    }

    pub fn get_by_key_and_vary_hash(conn: &Connection, key: &[u8; 32], vary_hash: &[u8; 32]) -> Result<Option<Index>> {
        let mut stmt = conn.prepare(
            "SELECT
                key,
                entry_type,
                file_id,
                offset,
                header_size,
                content_size,
                etag,
                expires_at,
                fetched_at,
                last_modified,
                max_age_seconds,
                vary,
                vary_hash,
                must_revalidate,
                no_cache,
                created_at
            FROM cache_index WHERE key = ?1 AND vary_hash = ?2",
        )?;
        let mut rows = stmt.query(params![key, vary_hash])?;

        if let Some(row) = rows.next()? {
            let entry_type: String = row.get(1)?;
            let entry = match entry_type.as_str() {
                "block" => IndexEntry::Block,
                "large" => IndexEntry::Large,
                _ => return Ok(None),
            };

            let expires_at = row.get::<usize, Option<i64>>(7)?;
            let vary = HttpCache::deserialize_vary(row.get::<usize, Option<String>>(11)?);
            let created_at = row.get::<usize, isize>(14)?;

            Ok(Some(Index {
                key: row.get(0)?,
                entry,
                file_id: row.get(2)?,
                offset: row.get(3)?,
                header_size: row.get(4)?,
                content_size: row.get(5)?,
                etag: row.get(6)?,
                expires_at,
                fetched_at: row.get(8)?,
                last_modified: row.get(9)?,
                max_age_seconds: row.get(10)?,
                vary,
                vary_hash: *vary_hash,
                must_revalidate: row.get(12)?,
                no_cache: row.get(13)?,
                created_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Deletes an index entry by its key, used when an entry is found to be expired or corrupted.
    pub fn delete_by_key(conn: &Connection, key: &[u8; 32]) -> Result<()> {
        conn.execute("DELETE FROM cache_index WHERE key = ?1", params![key])?;
        Ok(())
    }

    pub fn revalidate_by_key(
        conn: &Connection,
        key: &[u8; 32],
        fetched_at: u64,
        expires_at: Option<u64>,
    ) -> Result<()> {
        let fetched_at = fetched_at.min(i64::MAX as u64) as i64;
        let expires_at = expires_at.map(|v| v.min(i64::MAX as u64) as i64);

        conn.execute(
            "UPDATE cache_index
            SET fetched_at = ?2, expires_at = ?3
            WHERE key = ?1;",
            params![key, fetched_at, expires_at],
        )?;

        Ok(())
    }

    /// Updates the offset of a block entry in the index, used during compaction
    /// when live entries are relocated within a block file.
    pub fn update_block_offset(conn: &Connection, key: &[u8; 32], new_offset: u32, new_header_size: u32) -> Result<()> {
        conn.execute(
            "UPDATE cache_index SET offset = ?1, header_size = ?2 WHERE key = ?3 AND entry_type = 'block'",
            params![new_offset, new_header_size, key],
        )?;
        Ok(())
    }
}

impl Table for IndexTable {
    type Record = Index;

    fn create_table(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "BEGIN TRANSACTION;
            CREATE TABLE IF NOT EXISTS cache_index (
                key BLOB,
                entry_type TEXT NOT NULL,
                file_id INTEGER NOT NULL,
                offset INTEGER,
                header_size INTEGER,
                content_size INTEGER NOT NULL,
                etag TEXT,
                expires_at INTEGER,
                fetched_at INTEGER NOT NULL,
                last_modified INTEGER,
                max_age_seconds INTEGER,
                vary TEXT,
                vary_hash BLOB NOT NULL,
                must_revalidate INTEGER,
                no_cache INTEGER,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (key, vary)
            );
            CREATE INDEX IF NOT EXISTS entry_type_idx ON cache_index (entry_type);
            CREATE INDEX IF NOT EXISTS expires_at_idx ON cache_index (expires_at);
            CREATE INDEX IF NOT EXISTS file_id_idx ON cache_index (file_id);
            COMMIT;",
        )
    }

    fn insert(conn: &Connection, data: &Self::Record) -> Result<()> {
        let entry_type = match data.entry {
            IndexEntry::Block => "block",
            IndexEntry::Large => "large",
        };

        let vary = HttpCache::serialize_vary(&data.vary);

        conn.execute(
            "INSERT OR REPLACE INTO cache_index (
                key,
                entry_type,
                file_id,
                offset,
                header_size,
                content_size,
                etag,
                expires_at,
                fetched_at,
                last_modified,
                max_age_seconds,
                vary,
                vary_hash,
                must_revalidate,
                no_cache,
                created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                &data.key,
                entry_type,
                data.file_id,
                data.offset,
                data.header_size,
                data.content_size,
                data.etag,
                data.expires_at,
                data.fetched_at,
                data.last_modified,
                data.max_age_seconds,
                vary,
                data.vary_hash,
                data.must_revalidate,
                data.no_cache,
                data.created_at,
            ],
        )?;

        Ok(())
    }
}
