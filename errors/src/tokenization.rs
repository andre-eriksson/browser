use thiserror::Error;

/// Position in the source text for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

impl std::fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// Types of parse errors that can occur during CSS tokenization
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_position_display() {
        let pos = SourcePosition::new(5, 10, 42);
        assert_eq!(pos.to_string(), "line 5, column 10");
    }

    #[test]
    fn test_error_display() {
        let pos = SourcePosition::new(1, 7, 6);
        let error = CssTokenizationError::NewlineInString(pos);
        assert_eq!(
            error.to_string(),
            "unexpected newline in string at line 1, column 7"
        );
    }

    #[test]
    fn test_error_position() {
        let pos = SourcePosition::new(2, 3, 15);
        let error = CssTokenizationError::EofInComment(pos);
        assert_eq!(error.position(), pos);
    }

    #[test]
    fn test_all_error_variants() {
        let pos = SourcePosition::new(1, 1, 0);

        let errors = vec![
            CssTokenizationError::InvalidEscape(pos),
            CssTokenizationError::NewlineInString(pos),
            CssTokenizationError::EofInString(pos),
            CssTokenizationError::EofInUrl(pos),
            CssTokenizationError::EofInComment(pos),
            CssTokenizationError::InvalidCharacterInUrl(pos),
            CssTokenizationError::InvalidEscapeInUrl(pos),
        ];

        for error in errors {
            // Ensure all variants can be displayed
            let _ = error.to_string();
            // Ensure position can be retrieved
            assert_eq!(error.position(), pos);
        }
    }
}
