use http::StatusCode;
use thiserror::Error;

use http_policy::errors::PolicyError;
use http_types::errors::RequestError;

/// Errors related to network operations, preventing successful completion of a network request.
#[derive(Error, Debug, Clone)]
pub enum NetworkError {
    #[error("Connection timed out")]
    Timeout,

    #[error("Connection refused")]
    ConnectionRefused,

    #[error("Failed to decode URL: {0}")]
    Decode(String),

    #[error(transparent)]
    InvalidUrl(#[from] url::ParseError),

    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Maximum redirects exceeded")]
    MaxRedirectsExceeded,

    #[error("Unable to decode the HTTP request: {0}")]
    DecodingError(String),

    #[error("HTTP error: {0}")]
    HttpStatus(StatusCode),
}

/// Errors that can occur during the processing of a network request.
#[derive(Error, Debug, Clone)]
pub enum FetchError {
    #[error("Network request failed: {0}")]
    Network(#[from] NetworkError),

    #[error(transparent)]
    Policy(PolicyError),

    #[error(transparent)]
    Request(RequestError),

    #[error("Preflight request failed")]
    PreflightFailed,
}
