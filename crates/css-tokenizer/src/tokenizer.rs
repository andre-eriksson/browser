//! CSS Tokenizer implementation following CSS Syntax Module Level 3
//! <https://www.w3.org/TR/css-syntax-3/#tokenization>

use crate::errors::{CssTokenizationError, SourcePosition};
use crate::{
    consumers::token::consume_token,
    tokens::{CssToken, CssTokenKind},
};
use tracing::debug;

/// Input stream for the tokenizer
pub struct InputStream {
    /// Characters of the input
    chars: Vec<char>,

    /// Current position in the input
    pos: usize,

    /// Current character
    pub current: Option<char>,

    /// Current line
    line: usize,

    /// Current column
    column: usize,

    /// Previous line
    prev_line: usize,

    /// Previous column
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
    pub errors: Vec<CssTokenizationError>,

    /// Whether to collect position information for tokens
    pub collect_positions: bool,
}

impl CssTokenizer {
    /// Create a new tokenizer from the given input string
    ///
    /// # Arguments
    /// * `input` - The input CSS string to tokenize
    pub fn new(input: &str, collect_positions: bool) -> Self {
        let preprocessed_input = CssTokenizer::preprocess(input);

        CssTokenizer {
            stream: InputStream::new(&preprocessed_input),
            errors: Vec::new(),
            collect_positions,
        }
    }

    /// Record a parse error at the current position
    pub fn record_error(&mut self, error_fn: fn(SourcePosition) -> CssTokenizationError) {
        let position = self.stream.position();
        self.errors.push(error_fn(position));
    }

    /// Record a parse error at the position of the last consumed character
    /// Use this when the error is caused by a character that was just consumed
    pub fn record_error_at_current_char(&mut self, error_fn: fn(SourcePosition) -> CssTokenizationError) {
        let position = self.stream.prev_position();
        self.errors.push(error_fn(position));
    }

    /// Tokenize the given input string and return a vector of tokens
    ///
    /// # Arguments
    /// * `input` - The input CSS string to tokenize
    pub fn tokenize(input: &str, collect_positions: bool) -> Vec<CssToken> {
        let tokenizer = CssTokenizer::new(input, collect_positions);

        for error in &tokenizer.errors {
            debug!("CSS Tokenization error: {}", error);
        }

        tokenizer.collect()
    }

    /// Collect the current position if position tracking is enabled
    #[inline]
    pub fn collect_positions(tokenizer: &mut CssTokenizer) -> Option<SourcePosition> {
        if tokenizer.collect_positions {
            Some(tokenizer.stream.position())
        } else {
            None
        }
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

        if matches!(token.kind, CssTokenKind::Eof) {
            None
        } else {
            Some(token)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::CssTokenizationError;
    use crate::tokens::{HashType, NumericValue};

    fn has_errors(tokenizer: &CssTokenizer) -> bool {
        !tokenizer.errors.is_empty()
    }

    fn get_errors(tokenizer: &CssTokenizer) -> &Vec<CssTokenizationError> {
        &tokenizer.errors
    }

    #[test]
    fn test_preprocess() {
        assert_eq!(CssTokenizer::preprocess("a\r\nb"), "a\nb");
        assert_eq!(CssTokenizer::preprocess("a\rb"), "a\nb");
        assert_eq!(CssTokenizer::preprocess("a\x0Cb"), "a\nb");
        assert_eq!(CssTokenizer::preprocess("a\0b"), "a\u{FFFD}b");
    }

    #[test]
    fn test_error_newline_in_string() {
        let mut tokenizer = CssTokenizer::new("\"hello\nworld\"", true);
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(has_errors(&tokenizer));
        let errors = get_errors(&tokenizer);
        assert_eq!(errors.len(), 2);
        assert!(matches!(errors[0], CssTokenizationError::NewlineInString(_)));
        assert_eq!(errors[0].position().line, 1);
        assert_eq!(errors[0].position().column, 7);
        assert!(matches!(errors[1], CssTokenizationError::EofInString(_)));
    }

    #[test]
    fn test_error_eof_in_string() {
        let mut tokenizer = CssTokenizer::new("\"unterminated", true);
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(has_errors(&tokenizer));
        let errors = get_errors(&tokenizer);
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], CssTokenizationError::EofInString(_)));
    }

    #[test]
    fn test_error_eof_in_comment() {
        let mut tokenizer = CssTokenizer::new("/* unterminated comment", true);
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(has_errors(&tokenizer));
        let errors = get_errors(&tokenizer);
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], CssTokenizationError::EofInComment(_)));
    }

    #[test]
    fn test_error_invalid_escape() {
        let mut tokenizer = CssTokenizer::new("\\\n", true);
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(has_errors(&tokenizer));
        let errors = get_errors(&tokenizer);
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], CssTokenizationError::InvalidEscape(_)));
    }

    #[test]
    fn test_error_eof_in_url() {
        let mut tokenizer = CssTokenizer::new("url(unterminated", true);
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(has_errors(&tokenizer));
        let errors = get_errors(&tokenizer);
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], CssTokenizationError::EofInUrl(_)));
    }

    #[test]
    fn test_error_invalid_char_in_url() {
        let mut tokenizer = CssTokenizer::new("url(bad\"quote)", true);
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(has_errors(&tokenizer));
        let errors = get_errors(&tokenizer);
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], CssTokenizationError::InvalidCharacterInUrl(_)));
    }

    #[test]
    fn test_error_position_tracking() {
        let mut tokenizer = CssTokenizer::new(".foo {\n  color: \"bad\n", true);
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(has_errors(&tokenizer));
        let errors = get_errors(&tokenizer);
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], CssTokenizationError::NewlineInString(_)));
        assert_eq!(errors[0].position().line, 2);
        assert_eq!(errors[0].position().column, 14);
    }

    #[test]
    fn test_no_errors_on_valid_input() {
        let mut tokenizer = CssTokenizer::new(".foo { color: red; }", true);
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(!has_errors(&tokenizer));
        assert_eq!(get_errors(&tokenizer).len(), 0);
    }

    #[test]
    fn test_multiple_errors() {
        let mut tokenizer = CssTokenizer::new("\"bad\n \"also bad\n", true);
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(has_errors(&tokenizer));
        let errors = get_errors(&tokenizer);
        assert_eq!(errors.len(), 2);
        assert!(matches!(errors[0], CssTokenizationError::NewlineInString(_)));
        assert!(matches!(errors[1], CssTokenizationError::NewlineInString(_)));
    }

    #[test]
    fn test_take_errors() {
        let mut tokenizer = CssTokenizer::new("\"bad\n", true);
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(has_errors(&tokenizer));
        let errors = std::mem::take(&mut tokenizer.errors);
        assert_eq!(errors.len(), 1);
        assert!(!has_errors(&tokenizer));
    }

    #[test]
    fn test_empty_input() {
        let tokens = CssTokenizer::tokenize("", true);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let tokens = CssTokenizer::tokenize("   \t\n  ", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Whitespace));
    }

    #[test]
    fn test_single_tokens() {
        assert!(matches!(CssTokenizer::tokenize("(", true)[0].kind, CssTokenKind::OpenParen));
        assert!(matches!(CssTokenizer::tokenize(")", true)[0].kind, CssTokenKind::CloseParen));
        assert!(matches!(CssTokenizer::tokenize("{", true)[0].kind, CssTokenKind::OpenCurly));
        assert!(matches!(CssTokenizer::tokenize("}", true)[0].kind, CssTokenKind::CloseCurly));
        assert!(matches!(CssTokenizer::tokenize("[", true)[0].kind, CssTokenKind::OpenSquare));
        assert!(matches!(CssTokenizer::tokenize("]", true)[0].kind, CssTokenKind::CloseSquare));
        assert!(matches!(CssTokenizer::tokenize(":", true)[0].kind, CssTokenKind::Colon));
        assert!(matches!(CssTokenizer::tokenize(";", true)[0].kind, CssTokenKind::Semicolon));
        assert!(matches!(CssTokenizer::tokenize(",", true)[0].kind, CssTokenKind::Comma));
    }

    #[test]
    fn test_simple_identifiers() {
        let tokens = CssTokenizer::tokenize("div", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Ident(s) if s == "div"));
    }

    #[test]
    fn test_identifier_with_hyphen() {
        let tokens = CssTokenizer::tokenize("font-family", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Ident(s) if s == "font-family"));
    }

    #[test]
    fn test_identifier_starting_with_hyphen() {
        let tokens = CssTokenizer::tokenize("-webkit-transform", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Ident(s) if s == "-webkit-transform"));
    }

    #[test]
    fn test_identifier_with_double_hyphen() {
        let tokens = CssTokenizer::tokenize("--primary-color", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Ident(s) if s == "--primary-color"));
    }

    #[test]
    fn test_identifier_with_underscore() {
        let tokens = CssTokenizer::tokenize("_private_class", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Ident(s) if s == "_private_class"));
    }

    #[test]
    fn test_identifier_with_digits() {
        let tokens = CssTokenizer::tokenize("class123", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Ident(s) if s == "class123"));
    }

    #[test]
    fn test_identifier_unicode() {
        let tokens = CssTokenizer::tokenize("élément", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Ident(s) if s == "élément"));
    }

    #[test]
    fn test_identifier_with_escape() {
        let tokens = CssTokenizer::tokenize(r"\31 abc", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Ident(s) if s == "1abc"));
    }

    #[test]
    fn test_multiple_identifiers() {
        let tokens = CssTokenizer::tokenize("hello world", true);
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Ident(s) if s == "hello"));
        assert!(matches!(&tokens[1].kind, CssTokenKind::Whitespace));
        assert!(matches!(&tokens[2].kind, CssTokenKind::Ident(s) if s == "world"));
    }

    #[test]
    fn test_integer() {
        let tokens = CssTokenizer::tokenize("123", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Number(num) = &tokens[0].kind {
            assert_eq!(num.to_i64().unwrap(), 123);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_negative_integer() {
        let tokens = CssTokenizer::tokenize("-42", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Number(num) = &tokens[0].kind {
            assert_eq!(num.to_i64().unwrap(), -42);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_positive_integer_with_sign() {
        let tokens = CssTokenizer::tokenize("+100", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Number(num) = &tokens[0].kind {
            assert_eq!(num.to_i64().unwrap(), 100);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_decimal_number() {
        let tokens = CssTokenizer::tokenize("3.14", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Number(num) = &tokens[0].kind {
            assert!((num.to_f64() - 3.14).abs() < 0.0001);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_decimal_starting_with_dot() {
        let tokens = CssTokenizer::tokenize(".5", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Number(num) = &tokens[0].kind {
            assert!((num.to_f64() - 0.5).abs() < 0.0001);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_number_with_exponent() {
        let tokens = CssTokenizer::tokenize("1e10", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Number(num) = &tokens[0].kind {
            assert!((num.to_f64() - 1e10).abs() < 1e5);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_number_with_negative_exponent() {
        let tokens = CssTokenizer::tokenize("1e-5", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Number(num) = &tokens[0].kind {
            assert!((num.to_f64() - 1e-5).abs() < 1e-10);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_number_with_positive_exponent() {
        let tokens = CssTokenizer::tokenize("2E+3", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Number(num) = &tokens[0].kind {
            assert!((num.to_f64() - 2000.0).abs() < 0.0001);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_zero() {
        let tokens = CssTokenizer::tokenize("0", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Number(num) = &tokens[0].kind {
            assert_eq!(num.to_i64().unwrap(), 0);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_dimension_px() {
        let tokens = CssTokenizer::tokenize("100px", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Dimension { value, unit } = &tokens[0].kind {
            assert_eq!(value.to_f64(), 100.0);
            assert_eq!(unit, "px");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_dimension_em() {
        let tokens = CssTokenizer::tokenize("1.5em", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Dimension { value, unit } = &tokens[0].kind {
            assert!((value.to_f64() - 1.5).abs() < 0.0001);
            assert_eq!(unit, "em");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_dimension_rem() {
        let tokens = CssTokenizer::tokenize("2rem", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Dimension { value, unit } = &tokens[0].kind {
            assert_eq!(value.to_f64(), 2.0);
            assert_eq!(unit, "rem");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_dimension_deg() {
        let tokens = CssTokenizer::tokenize("45deg", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Dimension { value, unit } = &tokens[0].kind {
            assert_eq!(value.to_f64(), 45.0);
            assert_eq!(unit, "deg");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_dimension_ms() {
        let tokens = CssTokenizer::tokenize("300ms", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Dimension { value, unit } = &tokens[0].kind {
            assert_eq!(value.to_f64(), 300.0);
            assert_eq!(unit, "ms");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_dimension_s() {
        let tokens = CssTokenizer::tokenize("0.5s", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Dimension { value, unit } = &tokens[0].kind {
            assert!((value.to_f64() - 0.5).abs() < 0.0001);
            assert_eq!(unit, "s");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_dimension_vw() {
        let tokens = CssTokenizer::tokenize("100vw", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Dimension { value, unit } = &tokens[0].kind {
            assert_eq!(value.to_f64(), 100.0);
            assert_eq!(unit, "vw");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_dimension_negative() {
        let tokens = CssTokenizer::tokenize("-10px", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Dimension { value, unit } = &tokens[0].kind {
            assert_eq!(value.to_f64(), -10.0);
            assert_eq!(unit, "px");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_percentage() {
        let tokens = CssTokenizer::tokenize("50%", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Percentage(num) = &tokens[0].kind {
            assert_eq!(num.to_f64(), 50.0);
        } else {
            panic!("Expected Percentage token");
        }
    }

    #[test]
    fn test_percentage_decimal() {
        let tokens = CssTokenizer::tokenize("33.33%", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Percentage(num) = &tokens[0].kind {
            assert!((num.to_f64() - 33.33).abs() < 0.0001);
        } else {
            panic!("Expected Percentage token");
        }
    }

    #[test]
    fn test_percentage_negative() {
        let tokens = CssTokenizer::tokenize("-25%", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Percentage(num) = &tokens[0].kind {
            assert_eq!(num.to_f64(), -25.0);
        } else {
            panic!("Expected Percentage token");
        }
    }

    #[test]
    fn test_double_quoted_string() {
        let tokens = CssTokenizer::tokenize(r#""hello world""#, true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::String(s) if s == "hello world"));
    }

    #[test]
    fn test_single_quoted_string() {
        let tokens = CssTokenizer::tokenize("'hello world'", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::String(s) if s == "hello world"));
    }

    #[test]
    fn test_empty_string() {
        let tokens = CssTokenizer::tokenize(r#""""#, true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::String(s) if s.is_empty()));
    }

    #[test]
    fn test_string_with_escaped_quote() {
        let tokens = CssTokenizer::tokenize(r#""hello \"world\"""#, true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::String(s) if s == r#"hello "world""#));
    }

    #[test]
    fn test_string_with_escaped_newline() {
        let tokens = CssTokenizer::tokenize("\"hello\\\nworld\"", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::String(s) if s == "helloworld"));
    }

    #[test]
    fn test_string_with_hex_escape() {
        let tokens = CssTokenizer::tokenize(r#""\41 BC""#, true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::String(s) if s == "ABC"));
    }

    #[test]
    fn test_string_with_unicode_escape() {
        let tokens = CssTokenizer::tokenize(r#""\1F600""#, true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::String(s) if s == "😀"));
    }

    #[test]
    fn test_hash_id() {
        let tokens = CssTokenizer::tokenize("#header", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Hash { value, type_flag } = &tokens[0].kind {
            assert_eq!(value, "header");
            assert_eq!(*type_flag, HashType::Id);
        } else {
            panic!("Expected Hash token");
        }
    }

    #[test]
    fn test_hash_color_hex3() {
        let tokens = CssTokenizer::tokenize("#fff", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Hash { value, type_flag } = &tokens[0].kind {
            assert_eq!(value, "fff");
            assert_eq!(*type_flag, HashType::Id);
        } else {
            panic!("Expected Hash token");
        }
    }

    #[test]
    fn test_hash_color_hex6() {
        let tokens = CssTokenizer::tokenize("#ff00ff", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Hash { value, type_flag } = &tokens[0].kind {
            assert_eq!(value, "ff00ff");
            assert_eq!(*type_flag, HashType::Id);
        } else {
            panic!("Expected Hash token");
        }
    }

    #[test]
    fn test_hash_numeric_start() {
        let tokens = CssTokenizer::tokenize("#123", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Hash { value, type_flag } = &tokens[0].kind {
            assert_eq!(value, "123");
            assert_eq!(*type_flag, HashType::Unrestricted);
        } else {
            panic!("Expected Hash token");
        }
    }

    #[test]
    fn test_hash_alphanumeric() {
        let tokens = CssTokenizer::tokenize("#abc123", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Hash { value, type_flag } = &tokens[0].kind {
            assert_eq!(value, "abc123");
            assert_eq!(*type_flag, HashType::Id);
        } else {
            panic!("Expected Hash token");
        }
    }

    #[test]
    fn test_function_rgb() {
        let tokens = CssTokenizer::tokenize("rgb(", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Function(s) if s == "rgb"));
    }

    #[test]
    fn test_function_rgba() {
        let tokens = CssTokenizer::tokenize("rgba(", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Function(s) if s == "rgba"));
    }

    #[test]
    fn test_function_calc() {
        let tokens = CssTokenizer::tokenize("calc(", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Function(s) if s == "calc"));
    }

    #[test]
    fn test_function_var() {
        let tokens = CssTokenizer::tokenize("var(", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Function(s) if s == "var"));
    }

    #[test]
    fn test_function_complete_call() {
        let tokens = CssTokenizer::tokenize("rgb(255, 0, 0)", true);
        assert_eq!(tokens.len(), 9);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Function(s) if s == "rgb"));
        assert!(matches!(&tokens[1].kind, CssTokenKind::Number(_)));
        assert!(matches!(&tokens[2].kind, CssTokenKind::Comma));
        assert!(matches!(&tokens[3].kind, CssTokenKind::Whitespace));
        assert!(matches!(&tokens[4].kind, CssTokenKind::Number(_)));
        assert!(matches!(&tokens[5].kind, CssTokenKind::Comma));
        assert!(matches!(&tokens[6].kind, CssTokenKind::Whitespace));
        assert!(matches!(&tokens[7].kind, CssTokenKind::Number(_)));
        assert!(matches!(&tokens[8].kind, CssTokenKind::CloseParen));
    }

    #[test]
    fn test_url_unquoted() {
        let tokens = CssTokenizer::tokenize("url(image.png)", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Url(s) if s == "image.png"));
    }

    #[test]
    fn test_url_with_path() {
        let tokens = CssTokenizer::tokenize("url(/path/to/image.png)", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Url(s) if s == "/path/to/image.png"));
    }

    #[test]
    fn test_url_quoted_double() {
        let tokens = CssTokenizer::tokenize("url(\"image.png\")", true);
        assert!(tokens.len() >= 2);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Function(s) if s == "url"));
        assert!(matches!(&tokens[1].kind, CssTokenKind::String(s) if s == "image.png"));
    }

    #[test]
    fn test_url_quoted_single() {
        let tokens = CssTokenizer::tokenize("url('image.png')", true);
        assert!(tokens.len() >= 2);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Function(s) if s == "url"));
        assert!(matches!(&tokens[1].kind, CssTokenKind::String(s) if s == "image.png"));
    }

    #[test]
    fn test_url_with_whitespace() {
        let tokens = CssTokenizer::tokenize("url(  image.png  )", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Url(s) if s == "image.png"));
    }

    #[test]
    fn test_at_keyword_media() {
        let tokens = CssTokenizer::tokenize("@media", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::AtKeyword(s) if s == "media"));
    }

    #[test]
    fn test_at_keyword_import() {
        let tokens = CssTokenizer::tokenize("@import", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::AtKeyword(s) if s == "import"));
    }

    #[test]
    fn test_at_keyword_keyframes() {
        let tokens = CssTokenizer::tokenize("@keyframes", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::AtKeyword(s) if s == "keyframes"));
    }

    #[test]
    fn test_at_keyword_font_face() {
        let tokens = CssTokenizer::tokenize("@font-face", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::AtKeyword(s) if s == "font-face"));
    }

    #[test]
    fn test_at_keyword_supports() {
        let tokens = CssTokenizer::tokenize("@supports", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::AtKeyword(s) if s == "supports"));
    }

    #[test]
    fn test_at_keyword_charset() {
        let tokens = CssTokenizer::tokenize("@charset", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::AtKeyword(s) if s == "charset"));
    }

    #[test]
    fn test_at_not_keyword() {
        let tokens = CssTokenizer::tokenize("@ ", true);
        assert_eq!(tokens.len(), 2);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Delim('@')));
        assert!(matches!(&tokens[1].kind, CssTokenKind::Whitespace));
    }

    #[test]
    fn test_cdo() {
        let tokens = CssTokenizer::tokenize("<!--", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Cdo));
    }

    #[test]
    fn test_cdc() {
        let tokens = CssTokenizer::tokenize("-->", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Cdc));
    }

    #[test]
    fn test_cdo_cdc_in_context() {
        let tokens = CssTokenizer::tokenize("<!-- div { } -->", true);
        assert!(tokens.len() >= 5);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Cdo));
        assert!(matches!(&tokens[tokens.len() - 1].kind, CssTokenKind::Cdc));
    }

    #[test]
    fn test_delim_plus() {
        let tokens = CssTokenizer::tokenize("+ ", true);
        assert_eq!(tokens.len(), 2);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Delim('+')));
    }

    #[test]
    fn test_delim_dot_not_number() {
        let tokens = CssTokenizer::tokenize(".class", true);
        assert_eq!(tokens.len(), 2);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Delim('.')));
        assert!(matches!(&tokens[1].kind, CssTokenKind::Ident(s) if s == "class"));
    }

    #[test]
    fn test_delim_asterisk() {
        let tokens = CssTokenizer::tokenize("*", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Delim('*')));
    }

    #[test]
    fn test_delim_greater_than() {
        let tokens = CssTokenizer::tokenize(">", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Delim('>')));
    }

    #[test]
    fn test_delim_tilde() {
        let tokens = CssTokenizer::tokenize("~", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Delim('~')));
    }

    #[test]
    fn test_delim_pipe() {
        let tokens = CssTokenizer::tokenize("|", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Delim('|')));
    }

    #[test]
    fn test_delim_equals() {
        let tokens = CssTokenizer::tokenize("=", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Delim('=')));
    }

    #[test]
    fn test_delim_caret() {
        let tokens = CssTokenizer::tokenize("^", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Delim('^')));
    }

    #[test]
    fn test_delim_dollar() {
        let tokens = CssTokenizer::tokenize("$", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Delim('$')));
    }

    #[test]
    fn test_simple_comment() {
        let tokens = CssTokenizer::tokenize("/* comment */", true);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_comment_before_rule() {
        let tokens = CssTokenizer::tokenize("/* comment */ div", true);
        assert_eq!(tokens.len(), 2);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Whitespace));
        assert!(matches!(&tokens[1].kind, CssTokenKind::Ident(s) if s == "div"));
    }

    #[test]
    fn test_multiline_comment() {
        let tokens = CssTokenizer::tokenize("/* line 1\n   line 2\n   line 3 */", true);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_nested_comment_markers() {
        let tokens = CssTokenizer::tokenize("/* outer /* inner */ */", true);
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Whitespace));
        assert!(matches!(&tokens[1].kind, CssTokenKind::Delim('*')));
        assert!(matches!(&tokens[2].kind, CssTokenKind::Delim('/')));
    }

    #[test]
    fn test_comment_with_special_chars() {
        let tokens = CssTokenizer::tokenize("/* <>&\"' */", true);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_escape_single_char() {
        let tokens = CssTokenizer::tokenize(r"\.class", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Ident(s) if s == ".class"));
    }

    #[test]
    fn test_escape_hex_1_digit() {
        let tokens = CssTokenizer::tokenize(r"\A", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Ident(s) if s == "\n"));
    }

    #[test]
    fn test_escape_hex_6_digits() {
        let tokens = CssTokenizer::tokenize(r"\000041BC", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Ident(s) if s == "ABC"));
    }

    #[test]
    fn test_escape_followed_by_space() {
        let tokens = CssTokenizer::tokenize(r"\41 B", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Ident(s) if s == "AB"));
    }

    #[test]
    fn test_crlf_to_lf() {
        let tokens = CssTokenizer::tokenize("a\r\nb", true);
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Ident(s) if s == "a"));
        assert!(matches!(&tokens[1].kind, CssTokenKind::Whitespace));
        assert!(matches!(&tokens[2].kind, CssTokenKind::Ident(s) if s == "b"));
    }

    #[test]
    fn test_cr_to_lf() {
        let tokens = CssTokenizer::tokenize("a\rb", true);
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[1].kind, CssTokenKind::Whitespace));
    }

    #[test]
    fn test_ff_to_lf() {
        let tokens = CssTokenizer::tokenize("a\x0Cb", true);
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[1].kind, CssTokenKind::Whitespace));
    }

    #[test]
    fn test_null_to_replacement() {
        let tokens = CssTokenizer::tokenize("a\0b", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Ident(s) if s == "a\u{FFFD}b"));
    }

    #[test]
    fn test_simple_rule() {
        let tokens = CssTokenizer::tokenize("div { color: red; }", true);
        assert!(tokens.len() >= 9);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Ident(s) if s == "div"));
    }

    #[test]
    fn test_class_selector() {
        let tokens = CssTokenizer::tokenize(".container", true);
        assert_eq!(tokens.len(), 2);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Delim('.')));
        assert!(matches!(&tokens[1].kind, CssTokenKind::Ident(s) if s == "container"));
    }

    #[test]
    fn test_id_selector() {
        let tokens = CssTokenizer::tokenize("#main", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Hash { value, type_flag: HashType::Id } if value == "main"));
    }

    #[test]
    fn test_attribute_selector() {
        let tokens = CssTokenizer::tokenize("[data-attr=\"value\"]", true);
        assert!(tokens.len() >= 5);
        assert!(matches!(&tokens[0].kind, CssTokenKind::OpenSquare));
        assert!(matches!(&tokens[1].kind, CssTokenKind::Ident(s) if s == "data-attr"));
    }

    #[test]
    fn test_pseudo_class() {
        let tokens = CssTokenizer::tokenize(":hover", true);
        assert_eq!(tokens.len(), 2);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Colon));
        assert!(matches!(&tokens[1].kind, CssTokenKind::Ident(s) if s == "hover"));
    }

    #[test]
    fn test_pseudo_element() {
        let tokens = CssTokenizer::tokenize("::before", true);
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Colon));
        assert!(matches!(&tokens[1].kind, CssTokenKind::Colon));
        assert!(matches!(&tokens[2].kind, CssTokenKind::Ident(s) if s == "before"));
    }

    #[test]
    fn test_media_query() {
        let tokens = CssTokenizer::tokenize("@media screen and (min-width: 768px)", true);
        assert!(tokens.len() >= 8);
        assert!(matches!(&tokens[0].kind, CssTokenKind::AtKeyword(s) if s == "media"));
    }

    #[test]
    fn test_calc_expression() {
        let tokens = CssTokenizer::tokenize("calc(100% - 20px)", true);
        assert!(tokens.len() >= 6);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Function(s) if s == "calc"));
    }

    #[test]
    fn test_var_function() {
        let tokens = CssTokenizer::tokenize("var(--primary-color)", true);
        assert!(tokens.len() >= 3);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Function(s) if s == "var"));
        assert!(matches!(&tokens[1].kind, CssTokenKind::Ident(s) if s == "--primary-color"));
    }

    #[test]
    fn test_gradient() {
        let tokens = CssTokenizer::tokenize("linear-gradient(to right, red, blue)", true);
        assert!(tokens.len() >= 8);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Function(s) if s == "linear-gradient"));
    }

    #[test]
    fn test_important() {
        let tokens = CssTokenizer::tokenize("!important", true);
        assert_eq!(tokens.len(), 2);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Delim('!')));
        assert!(matches!(&tokens[1].kind, CssTokenKind::Ident(s) if s == "important"));
    }

    #[test]
    fn test_token_display_ident() {
        let token = CssTokenKind::Ident("hello".to_string());
        assert_eq!(format!("{}", token), "hello");
    }

    #[test]
    fn test_token_display_function() {
        let token = CssTokenKind::Function("rgb".to_string());
        assert_eq!(format!("{}", token), "rgb(");
    }

    #[test]
    fn test_token_display_at_keyword() {
        let token = CssTokenKind::AtKeyword("media".to_string());
        assert_eq!(format!("{}", token), "@media");
    }

    #[test]
    fn test_token_display_hash() {
        let token = CssTokenKind::Hash {
            value: "fff".to_string(),
            type_flag: HashType::Id,
        };
        assert_eq!(format!("{}", token), "#fff");
    }

    #[test]
    fn test_token_display_string() {
        let token = CssTokenKind::String("hello".to_string());
        assert_eq!(format!("{}", token), "\"hello\"");
    }

    #[test]
    fn test_token_display_url() {
        let token = CssTokenKind::Url("image.png".to_string());
        assert_eq!(format!("{}", token), "url(image.png)");
    }

    #[test]
    fn test_token_display_number() {
        let token = CssTokenKind::Number(NumericValue::from(42));
        assert_eq!(format!("{}", token), "42");
    }

    #[test]
    fn test_token_display_percentage() {
        let token = CssTokenKind::Percentage(NumericValue::from(50));
        assert_eq!(format!("{}", token), "50%");
    }

    #[test]
    fn test_token_display_dimension() {
        let token = CssTokenKind::Dimension {
            value: NumericValue::from(100),
            unit: "px".to_string(),
        };
        assert_eq!(format!("{}", token), "100px");
    }

    #[test]
    fn test_token_display_delim() {
        let token = CssTokenKind::Delim('*');
        assert_eq!(format!("{}", token), "*");
    }

    #[test]
    fn test_token_display_brackets() {
        assert_eq!(format!("{}", CssTokenKind::OpenParen), "(");
        assert_eq!(format!("{}", CssTokenKind::CloseParen), ")");
        assert_eq!(format!("{}", CssTokenKind::OpenCurly), "{");
        assert_eq!(format!("{}", CssTokenKind::CloseCurly), "}");
        assert_eq!(format!("{}", CssTokenKind::OpenSquare), "[");
        assert_eq!(format!("{}", CssTokenKind::CloseSquare), "]");
    }

    #[test]
    fn test_token_display_punctuation() {
        assert_eq!(format!("{}", CssTokenKind::Colon), ":");
        assert_eq!(format!("{}", CssTokenKind::Semicolon), ";");
        assert_eq!(format!("{}", CssTokenKind::Comma), ",");
    }

    #[test]
    fn test_token_display_whitespace() {
        assert_eq!(format!("{}", CssTokenKind::Whitespace), " ");
    }

    #[test]
    fn test_token_display_cdo_cdc() {
        assert_eq!(format!("{}", CssTokenKind::Cdo), "<!--");
        assert_eq!(format!("{}", CssTokenKind::Cdc), "-->");
    }

    #[test]
    fn test_token_display_eof() {
        assert_eq!(format!("{}", CssTokenKind::Eof), "");
    }

    #[test]
    fn test_unterminated_string_returns_bad_string() {
        let tokens = CssTokenizer::tokenize("\"unterminated", true);
        assert!(!tokens.is_empty());
        assert!(
            matches!(&tokens[0].kind, CssTokenKind::String(_)) || matches!(&tokens[0].kind, CssTokenKind::BadString)
        );
    }

    #[test]
    fn test_string_with_newline_returns_bad_string() {
        let tokens = CssTokenizer::tokenize("\"bad\nstring\"", true);
        assert!(!tokens.is_empty());
        assert!(matches!(&tokens[0].kind, CssTokenKind::BadString));
    }

    #[test]
    fn test_invalid_escape_at_top_level() {
        let tokens = CssTokenizer::tokenize("\\\n", true);
        assert!(!tokens.is_empty());
        assert!(matches!(&tokens[0].kind, CssTokenKind::Delim('\\')));
    }

    #[test]
    fn test_multiple_consecutive_numbers() {
        let tokens = CssTokenizer::tokenize("1 2 3", true);
        assert_eq!(tokens.len(), 5);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Number(n) if n.to_f64() == 1.0));
        assert!(matches!(&tokens[2].kind, CssTokenKind::Number(n) if n.to_f64() == 2.0));
        assert!(matches!(&tokens[4].kind, CssTokenKind::Number(n) if n.to_f64() == 3.0));
    }

    #[test]
    fn test_number_immediately_followed_by_ident() {
        let tokens = CssTokenizer::tokenize("10abc", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Dimension { unit, .. } if unit == "abc"));
    }

    #[test]
    fn test_negative_number_as_dimension() {
        let tokens = CssTokenizer::tokenize("-5em", true);
        assert_eq!(tokens.len(), 1);
        if let CssTokenKind::Dimension { value, unit } = &tokens[0].kind {
            assert_eq!(value.to_f64(), -5.0);
            assert_eq!(unit, "em");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_very_long_identifier() {
        let long_ident = "a".repeat(1000);
        let tokens = CssTokenizer::tokenize(&long_ident, true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Ident(s) if s == &long_ident));
    }

    #[test]
    fn test_very_large_number() {
        let tokens = CssTokenizer::tokenize("999999999999999999999", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Number(_)));
    }

    #[test]
    fn test_scientific_notation_edge_cases() {
        let tokens = CssTokenizer::tokenize("1eabc", true);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, CssTokenKind::Dimension { unit, .. } if unit == "eabc"));
    }
}
