use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum CookieParsingError {
    #[error("Cookie must have at least a name=value pair")]
    InvalidCookie,

    #[error("Invalid date format: {0}")]
    DateError(String),

    #[error("Invalid time format: {0}")]
    TimeError(String),

    #[error("Unable to parse {0}: {1}")]
    Parsing(String, String),

    #[error("{prefix} prefixed cookies must {message}")]
    PrefixMismatch { prefix: String, message: String },
}
