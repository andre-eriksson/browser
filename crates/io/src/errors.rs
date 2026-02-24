//! This module defines the `AssetError` enum, which represents various errors
//! that can occur when loading assets in the application.

#[cfg(feature = "network")]
use network::errors::RequestError;
use thiserror::Error;

/// AssetError represents errors that can occur when loading assets.
#[derive(Error, Debug)]
pub enum AssetError {
    #[error("Asset not found: {0}")]
    NotFound(String),

    #[error("Asset load failed: {0}")]
    LoadFailed(String),

    #[error("Asset write failed: {0}")]
    WriteFailed(String),

    #[cfg(feature = "network")]
    #[error("Remote request failed: {0}")]
    RemoteFailed(#[from] RequestError),

    #[error("Unsupported protocol: {0}")]
    UnsupportedProtocol(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Resource is unavailable, check if the {0} directory exists and is writable.")]
    Unavailable(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}
