use rusqlite::{Connection, Result};

pub trait Database {
    /// Opens a connection to the database.
    fn open() -> Result<Connection>;
}

pub trait Table {
    type Record;

    /// Creates the table in the database if it doesn't exist.
    fn create_table(conn: &Connection) -> Result<()>;

    /// Inserts a record into the table.
    fn insert(conn: &Connection, data: &Self::Record) -> Result<()>;
}
