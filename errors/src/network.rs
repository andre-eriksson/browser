use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum HttpError {
    #[error("Header parse error: {0}")]
    HeaderParseError(String),

    #[error("Invalid URL: {0}")]
    InvalidURL(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

#[derive(Error, Debug, Clone)]
pub enum NetworkError {
    #[error("Network request failed: {0}")]
    RequestFailed(String),

    #[error("CORS error: {0}")]
    CORSError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),
}
