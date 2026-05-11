//! Errors related to cache operations.

use network::{HeaderMap, response::Response};
use thiserror::Error;

/// Represents the result of a cache read operation, indicating whether it was a hit or a miss.
#[derive(Debug, Clone)]
pub enum CacheRead {
    Hit(Response),
    RequiresRevalidation {
        /// The stale data to use if the server responds with 304 Not Modified
        stale_data: Response,
        /// The headers to attach to the outbound request (e.g., If-None-Match)
        revalidation_headers: HeaderMap,
    },
    Miss,
}

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
