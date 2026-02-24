use rusqlite::{Connection, Result};

pub trait Database {
    fn open() -> Result<Connection>;
}

pub trait Table {
    type Record;

    fn create_table(conn: &Connection) -> Result<()>;
    fn insert(conn: &Connection, data: &Self::Record) -> Result<()>;
}
