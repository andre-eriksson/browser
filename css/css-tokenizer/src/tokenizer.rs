//! CSS Tokenizer implementation following CSS Syntax Module Level 3
//! <https://www.w3.org/TR/css-syntax-3/#tokenization>

use crate::{consumers::token::consume_token, tokens::CssToken};
use errors::tokenization::{CssTokenizationError, SourcePosition};

/// Input stream for the tokenizer
pub struct InputStream {
    /// Characters of the input
    chars: Vec<char>,

    /// Current position in the input
    pos: usize,

    /// Current character
    pub current: Option<char>,

    // For tracking line and column
    line: usize,
    column: usize,

    // For reconsume
    prev_line: usize,
    prev_column: usize,
}

impl InputStream {
    /// Create a new input stream from the given string
    pub fn new(input: &str) -> Self {
        InputStream {
            chars: input.chars().collect(),
            pos: 0,
            current: None,
            line: 1,
            column: 1,
            prev_line: 1,
            prev_column: 1,
        }
    }

    /// Peek at the current character without consuming it
    pub fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// Peek at the character at the given offset without consuming it
    pub fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    /// Consume the current character and advance the position
    pub fn consume(&mut self) -> Option<char> {
        if self.pos < self.chars.len() {
            self.prev_line = self.line;
            self.prev_column = self.column;

            self.current = Some(self.chars[self.pos]);
            self.pos += 1;

            if self.current == Some('\n') {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }

            self.current
        } else {
            self.current = None;
            None
        }
    }

    /// Reconsume the current character by moving the position back by one
    pub fn reconsume(&mut self) {
        if self.pos > 0 && self.current.is_some() {
            self.pos -= 1;
            self.line = self.prev_line;
            self.column = self.prev_column;
        }
    }

    /// Get the current source position
    pub fn position(&self) -> SourcePosition {
        SourcePosition {
            line: self.line,
            column: self.column,
            offset: self.pos,
        }
    }

    /// Get the position of the previously consumed character
    /// This is useful for recording errors at the position of the character
    /// that caused the error, rather than the position after consuming it.
    pub fn prev_position(&self) -> SourcePosition {
        SourcePosition {
            line: self.prev_line,
            column: self.prev_column,
            offset: if self.pos > 0 { self.pos - 1 } else { 0 },
        }
    }
}

/// CSS Tokenizer following CSS Syntax Module Level 3
pub struct CssTokenizer {
    /// Input stream for the tokenizer
    pub stream: InputStream,

    /// Collected parse errors
    errors: Vec<CssTokenizationError>,
}

impl CssTokenizer {
    /// Create a new tokenizer from the given input string
    ///
    /// # Arguments
    /// * `input` - The input CSS string to tokenize
    fn new(input: &str) -> Self {
        let preprocessed_input = CssTokenizer::preprocess(input);

        CssTokenizer {
            stream: InputStream::new(&preprocessed_input),
            errors: Vec::new(),
        }
    }

    /// Record a parse error at the current position
    pub fn record_error(&mut self, error_fn: fn(SourcePosition) -> CssTokenizationError) {
        let position = self.stream.position();
        self.errors.push(error_fn(position));
    }

    /// Record a parse error at the position of the last consumed character
    /// Use this when the error is caused by a character that was just consumed
    pub fn record_error_at_current_char(
        &mut self,
        error_fn: fn(SourcePosition) -> CssTokenizationError,
    ) {
        let position = self.stream.prev_position();
        self.errors.push(error_fn(position));
    }

    /// Get all recorded parse errors
    pub fn get_errors(&self) -> &[CssTokenizationError] {
        &self.errors
    }

    /// Take all recorded parse errors, leaving the handler empty
    pub fn take_errors(&mut self) -> Vec<CssTokenizationError> {
        std::mem::take(&mut self.errors)
    }

    /// Check if any parse errors were recorded
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Tokenize the given input string and return a vector of tokens
    ///
    /// # Arguments
    /// * `input` - The input CSS string to tokenize
    pub fn tokenize(input: &str) -> Vec<CssToken> {
        let tokenizer = CssTokenizer::new(input);

        tokenizer.collect()
    }

    /// Preprocess the input string according to the CSS specification (§3.3)
    ///
    /// # Arguments
    /// * `input` - The input CSS string to preprocess
    ///
    /// # Behavior
    /// * Replace CRLF (`\r\n`) with LF (`\n`)
    /// * Replace CR (`\r`) with LF (`\n`)
    /// * Replace FF (`\x0C`) with LF (`\n`)
    /// * Replace NULL (`\0`) with the REPLACEMENT CHARACTER (`\u{FFFD}`)
    ///
    /// # Note
    /// Surrogate code points are discarded by Rust's `char` type, so no special handling is needed.
    ///
    /// # Returns
    /// A new `String` with the preprocessed content
    fn preprocess(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    result.push('\n');
                }
                '\x0C' => {
                    result.push('\n');
                }
                '\0' => {
                    result.push('\u{FFFD}');
                }
                _ => result.push(c),
            }
        }

        result
    }
}

impl Iterator for CssTokenizer {
    type Item = CssToken;

    fn next(&mut self) -> Option<Self::Item> {
        let token = consume_token(self);

        if matches!(token, CssToken::Eof) {
            None
        } else {
            Some(token)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use errors::tokenization::CssTokenizationError;

    #[test]
    fn test_preprocess() {
        // Test CRLF -> LF
        assert_eq!(CssTokenizer::preprocess("a\r\nb"), "a\nb");

        // Test CR -> LF
        assert_eq!(CssTokenizer::preprocess("a\rb"), "a\nb");

        // Test FF -> LF
        assert_eq!(CssTokenizer::preprocess("a\x0Cb"), "a\nb");

        // Test NULL -> REPLACEMENT CHARACTER
        assert_eq!(CssTokenizer::preprocess("a\0b"), "a\u{FFFD}b");
    }

    #[test]
    fn test_error_newline_in_string() {
        // Input: "hello\nworld" - the newline causes BadString, then world" continues
        // which results in ident 'world' and then an unterminated string '"'
        let mut tokenizer = CssTokenizer::new("\"hello\nworld\"");
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(tokenizer.has_errors());
        let errors = tokenizer.get_errors();
        // We get 2 errors: newline in first string, and EOF in the trailing quote
        assert_eq!(errors.len(), 2);
        assert!(matches!(
            errors[0],
            CssTokenizationError::NewlineInString(_)
        ));
        // Error should be at the position of the newline (line 1, column 7)
        assert_eq!(errors[0].position().line, 1);
        assert_eq!(errors[0].position().column, 7);
        // Second error is EOF in the unterminated string at the end
        assert!(matches!(errors[1], CssTokenizationError::EofInString(_)));
    }

    #[test]
    fn test_error_eof_in_string() {
        let mut tokenizer = CssTokenizer::new("\"unterminated");
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(tokenizer.has_errors());
        let errors = tokenizer.get_errors();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], CssTokenizationError::EofInString(_)));
    }

    #[test]
    fn test_error_eof_in_comment() {
        let mut tokenizer = CssTokenizer::new("/* unterminated comment");
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(tokenizer.has_errors());
        let errors = tokenizer.get_errors();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], CssTokenizationError::EofInComment(_)));
    }

    #[test]
    fn test_error_invalid_escape() {
        let mut tokenizer = CssTokenizer::new("\\\n");
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(tokenizer.has_errors());
        let errors = tokenizer.get_errors();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], CssTokenizationError::InvalidEscape(_)));
    }

    #[test]
    fn test_error_eof_in_url() {
        let mut tokenizer = CssTokenizer::new("url(unterminated");
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(tokenizer.has_errors());
        let errors = tokenizer.get_errors();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], CssTokenizationError::EofInUrl(_)));
    }

    #[test]
    fn test_error_invalid_char_in_url() {
        let mut tokenizer = CssTokenizer::new("url(bad\"quote)");
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(tokenizer.has_errors());
        let errors = tokenizer.get_errors();
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0],
            CssTokenizationError::InvalidCharacterInUrl(_)
        ));
    }

    #[test]
    fn test_error_position_tracking() {
        let mut tokenizer = CssTokenizer::new(".foo {\n  color: \"bad\n");
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(tokenizer.has_errors());
        let errors = tokenizer.get_errors();
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0],
            CssTokenizationError::NewlineInString(_)
        ));
        // The newline is on line 2, column 14 (after "bad)
        assert_eq!(errors[0].position().line, 2);
        assert_eq!(errors[0].position().column, 14);
    }

    #[test]
    fn test_no_errors_on_valid_input() {
        let mut tokenizer = CssTokenizer::new(".foo { color: red; }");
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(!tokenizer.has_errors());
        assert_eq!(tokenizer.get_errors().len(), 0);
    }

    #[test]
    fn test_multiple_errors() {
        let mut tokenizer = CssTokenizer::new("\"bad\n \"also bad\n");
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(tokenizer.has_errors());
        let errors = tokenizer.get_errors();
        assert_eq!(errors.len(), 2);
        assert!(matches!(
            errors[0],
            CssTokenizationError::NewlineInString(_)
        ));
        assert!(matches!(
            errors[1],
            CssTokenizationError::NewlineInString(_)
        ));
    }

    #[test]
    fn test_take_errors() {
        let mut tokenizer = CssTokenizer::new("\"bad\n");
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(tokenizer.has_errors());
        let errors = tokenizer.take_errors();
        assert_eq!(errors.len(), 1);
        assert!(!tokenizer.has_errors());
    }
}
