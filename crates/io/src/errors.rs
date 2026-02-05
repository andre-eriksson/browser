use network::errors::RequestError;
use thiserror::Error;

/// AssetError represents errors that can occur when loading assets.
#[derive(Error, Debug)]
pub enum AssetError {
    #[error("Asset not found: {0}")]
    NotFound(String),

    #[error("Asset load failed: {0}")]
    LoadFailed(String),

    #[error("Remote request failed: {0}")]
    RemoteFailed(#[from] RequestError),

    #[error("The resource: '{0}' is incompatible with: {1}")]
    IncompatibleBackend(String, String),
}
