use css_cssom::{ComponentValue, CssTokenKind};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CssValueError {
    #[error("Unexpected end of input")]
    UnexpectedEndOfInput,

    #[error("Unexpected remaining input")]
    UnexpectedRemainingInput,

    #[error("Expected a component value")]
    ExpectedComponentValue,

    #[error("Invalid component value: {0}")]
    InvalidComponentValue(ComponentValue),

    #[error("Invalid function: {0}")]
    InvalidFunction(String),

    #[error("Invalid token: {0:?}")]
    InvalidToken(CssTokenKind),

    #[error("Invalid value: {0}")]
    InvalidValue(String),

    #[error("Invalid unit: {0}")]
    InvalidUnit(String),
}
