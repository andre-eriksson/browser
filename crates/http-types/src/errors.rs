use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum RequestError {
    #[error(transparent)]
    InvalidUrl(#[from] url::ParseError),

    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Maximum redirects exceeded")]
    MaxRedirectsExceeded,
}
