use rusqlite::{Connection, Result};
use storage::paths::{create_paths, get_data_path};

#[derive(Debug, Clone, Copy)]
pub struct Database {
    domain: Domain,
}

impl Database {
    pub fn new(domain: Domain) -> Self {
        Database { domain }
    }

    pub fn open(&self) -> Result<Connection> {
        let base_path = get_data_path().expect("Failed to get data path");
        let _ = create_paths(&base_path);

        let path = base_path.join(self.domain.filename());

        Connection::open(path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Domain {
    Cookies,
    History,
    Bookmarks,
}

impl Domain {
    fn filename(&self) -> &str {
        match self {
            Domain::Cookies => "cookies.db",
            Domain::History => "history.db",
            Domain::Bookmarks => "bookmarks.db",
        }
    }
}

pub trait Table {
    type Record;

    fn create_table(conn: &Connection) -> Result<()>;
    fn insert(conn: &Connection, data: &Self::Record) -> Result<()>;
}
