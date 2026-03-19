use rusqlite::{Connection, Result};

pub trait Database {
    /// Opens a connection to the database.
    ///
    /// # Errors
    /// * If the database file cannot be opened or created
    fn open() -> Result<Connection>;
}

pub trait Table {
    type Record;

    /// Creates the table in the database if it doesn't exist.
    ///
    /// # Arguments
    /// * `conn` - A reference to the database connection
    ///
    /// # Errors
    /// * If the table cannot be created in the database
    fn create_table(conn: &Connection) -> Result<()>;

    /// Inserts a record into the table.
    ///
    /// # Arguments
    /// * `conn` - A reference to the database connection
    /// * `data` - The record to be inserted into the table
    ///
    /// # Errors
    /// * If the record cannot be inserted into the table
    fn insert(conn: &Connection, data: &Self::Record) -> Result<()>;
}
