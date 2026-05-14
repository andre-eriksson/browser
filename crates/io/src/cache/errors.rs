//! Errors related to cache operations.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("cache directory not found, check if the cache directory exists and is writable.")]
    CacheDirectoryNotFound,

    #[error(transparent)]
    Serialization(#[from] postcard::Error),

    #[error(transparent)]
    Database(#[from] rusqlite::Error),

    #[error("couldn't get a lock on the database.")]
    DatabaseLock,

    #[error("{0}")]
    Read(String),

    #[error("{0}")]
    Write(String),

    #[error("cache index is corrupted")]
    CorruptedIndex,

    #[error("cache header is corrupted")]
    CorruptedHeader,

    #[error("cache block is corrupted")]
    CorruptedBlock,

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
