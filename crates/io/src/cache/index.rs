use database::{Database, Table};
use rusqlite::{Connection, Result, params};
use storage::paths::get_cache_path;

const IDX_DATABASE: &str = "resources/index.db";

pub struct Index {
    pub key: [u8; 32],
    pub entry: IndexEntry,
    pub file_id: u32,
    pub offset: Option<u32>,
    pub header_size: Option<u32>,
    pub content_size: u32,
    pub expires_at: Option<u64>,
    pub created_at: u64,
    pub vary: Option<String>,
}

pub struct IndexDatabase;
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

    pub fn delete_by_key(conn: &Connection, key: &[u8; 32]) -> Result<()> {
        conn.execute("DELETE FROM cache_index WHERE key = ?1", params![key])?;
        Ok(())
    }
}

impl Database for IndexDatabase {
    fn open() -> Result<Connection> {
        let path = get_cache_path()
            .ok_or_else(|| rusqlite::Error::InvalidPath("Cache path not found".into()))?
            .join(IDX_DATABASE);

        std::fs::create_dir_all(path.parent().unwrap())
            .ok()
            .ok_or_else(|| {
                rusqlite::Error::InvalidPath("Failed to create cache directory".into())
            })?;

        let conn = Connection::open(path)?;

        conn.execute_batch("PRAGMA journal_mode = WAL;")?;

        IndexTable::create_table(&conn)?;

        Ok(conn)
    }
}

pub enum IndexEntry {
    Block,
    Large,
}

impl Table for IndexTable {
    type Record = Index;

    fn create_table(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "BEGIN;
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
