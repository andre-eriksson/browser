use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CssParsingError {
    #[error("Unexpected end of input")]
    UnexpectedEndOfInput,

    #[error("Unexpected token | Expected: {0}, found: {1}")]
    UnexpectedToken(String, String),

    #[error("Unexpected tokens after {0}")]
    UnexpectedTokensAfter(String),

    #[error("Failed to parse: {0}")]
    ParseError(String),
}
