//! This module defines the `Index` struct and related database operations for managing cache entries in
//! the browser's caching system. It provides functionality to store metadata about cached resources,
//! including their location in block files or large files, expiration times, and other relevant information.
//! The `IndexDatabase` struct implements the `Database` trait to handle database connections and schema
//! management, while the `IndexTable` struct implements the `Table` trait to manage CRUD operations on
//! the index entries.

use database::{Database, Table};
use rusqlite::{Connection, Result, params};
use storage::paths::get_cache_path;

/// Path to the SQLite database file that stores the cache index entries.
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
pub struct Index {
    /// The SHA-256 hash of the cached resource's URL, used as the key for lookup in the index.
    pub key: [u8; 32],

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

    /// The UNIX timestamp (in seconds) when the cached entry expires, used to determine if the entry is still valid.
    pub expires_at: Option<u64>,

    /// The UNIX timestamp (in seconds) when the cached entry was created, used for cache management and eviction policies.
    pub created_at: u64,

    /// The headers associated with the cached entry that affect its cache key, derived from the Vary header, used for cache validation.
    pub vary: Option<String>,
}

/// Database interface for managing cache index entries, providing methods to open the database connection and ensure the schema is set up correctly.
pub struct IndexDatabase;

/// Table interface for managing cache index entries, providing methods to create the index table and perform CRUD operations on the entries.
pub struct IndexTable;

impl IndexTable {
    /// Retrieves an index entry by its key, ensuring it is not expired.
    pub fn get_by_key(conn: &Connection, key: &[u8; 32]) -> Result<Option<Index>> {
        let mut stmt = conn.prepare("SELECT key, entry_type, file_id, offset, header_size, content_size, expires_at, created_at, vary FROM cache_index WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;

        if let Some(row) = rows.next()? {
            let entry_type: String = row.get(1)?;
            let entry = match entry_type.as_str() {
                "block" => IndexEntry::Block,
                "large" => IndexEntry::Large,
                _ => return Ok(None),
            };

            let expires_at = row.get::<usize, Option<i64>>(6)?.map(|e| e as u64);
            let created_at = row.get::<usize, i64>(7)? as u64;

            Ok(Some(Index {
                key: row.get(0)?,
                entry,
                file_id: row.get(2)?,
                offset: row.get(3)?,
                header_size: row.get(4)?,
                content_size: row.get(5)?,
                expires_at,
                created_at,
                vary: row.get(8)?,
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

    /// Updates the offset of a block entry in the index, used during compaction
    /// when live entries are relocated within a block file.
    pub fn update_block_offset(
        conn: &Connection,
        key: &[u8; 32],
        new_offset: u32,
        new_header_size: u32,
    ) -> Result<()> {
        conn.execute(
            "UPDATE cache_index SET offset = ?1, header_size = ?2 WHERE key = ?3 AND entry_type = 'block'",
            params![new_offset, new_header_size, key],
        )?;
        Ok(())
    }
}

impl Database for IndexDatabase {
    fn open() -> Result<Connection> {
        let path = get_cache_path()
            .ok_or_else(|| rusqlite::Error::InvalidPath("Cache path not found".into()))?
            .join(IDX_DATABASE);

        std::fs::create_dir_all(path.parent().unwrap())
            .map_err(|_| rusqlite::Error::InvalidPath("Failed to create cache directory".into()))?;

        let conn = Connection::open(path)?;

        conn.execute_batch("PRAGMA journal_mode = WAL;")?;

        IndexTable::create_table(&conn)?;

        Ok(conn)
    }
}

impl Table for IndexTable {
    type Record = Index;

    fn create_table(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "BEGIN TRANSACTION;
            CREATE TABLE IF NOT EXISTS cache_index (
                key BLOB PRIMARY KEY,
                entry_type TEXT NOT NULL,
                file_id INTEGER NOT NULL,
                offset INTEGER,
                header_size INTEGER,
                content_size INTEGER NOT NULL,
                expires_at INTEGER,
                created_at INTEGER NOT NULL,
                vary TEXT
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

        conn.execute(
            "INSERT OR REPLACE INTO cache_index (key, entry_type, file_id, offset, header_size, content_size, expires_at, created_at, vary)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &data.key,
                entry_type,
                data.file_id,
                data.offset,
                data.header_size,
                data.content_size,
                data.expires_at.map(|e| e as i64),
                data.created_at as i64,
                data.vary
            ],
        )?;

        Ok(())
    }
}
