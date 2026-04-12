//! This module defines the `AssetError` enum, which represents various errors
//! that can occur when loading assets in the application.

use thiserror::Error;

/// AssetError represents errors that can occur when loading assets.
#[derive(Error, Debug, Clone)]
pub enum ResourceError {
    #[error("asset not found: {0}")]
    NotFound(String),

    #[error("failed to read asset: {0}")]
    Io(String),

    #[error("unsupported protocol: {0}")]
    UnsupportedProtocol(String),

    #[error("unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("invalid path: {0}")]
    InvalidPath(String),
}
