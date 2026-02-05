use thiserror::Error;

/// Errors related to network operations, preventing successful completion of a network request.
#[derive(Error, Debug, Clone)]
pub enum NetworkError {
    #[error("Network error: {0}")]
    RuntimeError(String),

    #[error("Connection timed out")]
    Timeout,

    #[error("Connection refused")]
    ConnectionRefused,

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Maximum redirects exceeded")]
    MaxRedirectsExceeded,
}

/// Errors that can occur during the processing of a network request.
#[derive(Error, Debug, Clone)]
pub enum RequestError {
    #[error("Network request failed: {0}")]
    Network(#[from] NetworkError),

    #[error("Invalid HTTP method: {0}")]
    InvalidMethod(String),

    #[error("Request body is empty")]
    EmptyBody,

    #[error("CORS preflight request failed")]
    PreflightFailed,

    #[error("CORS error: {0}")]
    CorsViolation(String),

    #[error("CSP violation: {0}")]
    CspViolation(String),

    #[error("Request blocked by policy: {0}")]
    BlockedByPolicy(String),
}
