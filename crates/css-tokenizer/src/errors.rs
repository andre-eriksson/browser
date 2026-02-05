use std::fmt::Display;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourcePosition {
    /// Line number
    pub line: usize,

    /// Column number
    pub column: usize,

    /// Byte offset from the start of the input
    pub offset: usize,
}

impl SourcePosition {
    /// Create a new source position
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }
}

impl Display for SourcePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CssTokenizationError {
    /// A backslash at the end of input or followed by a newline (invalid escape)
    #[error("invalid escape sequence at {0}")]
    InvalidEscape(SourcePosition),

    /// A string token contains an unescaped newline
    #[error("unexpected newline in string at {0}")]
    NewlineInString(SourcePosition),

    /// EOF reached before the end of a string
    #[error("unexpected end of file in string at {0}")]
    EofInString(SourcePosition),

    /// EOF reached before the end of a URL
    #[error("unexpected end of file in URL at {0}")]
    EofInUrl(SourcePosition),

    /// EOF reached before the end of a comment
    #[error("unexpected end of file in comment at {0}")]
    EofInComment(SourcePosition),

    /// Invalid character in URL (unquoted URL contains whitespace, quotes, or non-printable chars)
    #[error("invalid character in URL at {0}")]
    InvalidCharacterInUrl(SourcePosition),

    /// Invalid escape sequence in URL
    #[error("invalid escape sequence in URL at {0}")]
    InvalidEscapeInUrl(SourcePosition),
}

impl CssTokenizationError {
    /// Get the source position where the error occurred
    pub fn position(&self) -> SourcePosition {
        match self {
            CssTokenizationError::InvalidEscape(pos) => *pos,
            CssTokenizationError::NewlineInString(pos) => *pos,
            CssTokenizationError::EofInString(pos) => *pos,
            CssTokenizationError::EofInUrl(pos) => *pos,
            CssTokenizationError::EofInComment(pos) => *pos,
            CssTokenizationError::InvalidCharacterInUrl(pos) => *pos,
            CssTokenizationError::InvalidEscapeInUrl(pos) => *pos,
        }
    }
}
