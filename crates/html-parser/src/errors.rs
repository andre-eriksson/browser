use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum HtmlParsingError {
    #[error("HTML parsing has already finished")]
    AlreadyFinished,

    #[error("Parser is not blocked waiting for {0}")]
    InvalidBlockReason(String),

    #[error("Unable to read from stream: {0}")]
    UnableToReadStream(String),

    #[error("Malformed document: {0}")]
    MalformedDocument(String),

    #[error("Unexpected UTF-8 error: {0}")]
    UnexpectedUtf8Error(String),
}
