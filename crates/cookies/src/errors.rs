use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum ParsingError {
    #[error("must have at least a name=value pair")]
    InvalidCookie,

    #[error("invalid date format: {0}")]
    Date(String),

    #[error("invalid time format: {0}")]
    Time(String),

    #[error("unable to parse {0}: {1}")]
    Parsing(String, String),

    #[error("{prefix} prefixed cookies must {message}")]
    PrefixMismatch { prefix: String, message: String },
}
