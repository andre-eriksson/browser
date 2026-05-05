use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum HtmlParsingError {
    #[error("unable to read from stream: {0}")]
    UnableToReadStream(String),

    #[error("malformed document: {0}")]
    MalformedDocument(String),

    #[error("unexpected UTF-8 error: {0}")]
    UnexpectedUtf8Error(String),
}
