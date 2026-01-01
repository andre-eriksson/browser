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
    use crate::tokens::{HashType, NumberType, NumericValue};
    use errors::tokenization::CssTokenizationError;

    #[test]
    fn test_preprocess() {
        assert_eq!(CssTokenizer::preprocess("a\r\nb"), "a\nb");
        assert_eq!(CssTokenizer::preprocess("a\rb"), "a\nb");
        assert_eq!(CssTokenizer::preprocess("a\x0Cb"), "a\nb");
        assert_eq!(CssTokenizer::preprocess("a\0b"), "a\u{FFFD}b");
    }

    #[test]
    fn test_error_newline_in_string() {
        let mut tokenizer = CssTokenizer::new("\"hello\nworld\"");
        let _tokens: Vec<_> = tokenizer.by_ref().collect();

        assert!(tokenizer.has_errors());
        let errors = tokenizer.get_errors();
        assert_eq!(errors.len(), 2);
        assert!(matches!(
            errors[0],
            CssTokenizationError::NewlineInString(_)
        ));
        assert_eq!(errors[0].position().line, 1);
        assert_eq!(errors[0].position().column, 7);
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

    #[test]
    fn test_empty_input() {
        let tokens = CssTokenizer::tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let tokens = CssTokenizer::tokenize("   \t\n  ");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], CssToken::Whitespace));
    }

    #[test]
    fn test_single_tokens() {
        assert!(matches!(
            CssTokenizer::tokenize("(")[0],
            CssToken::OpenParen
        ));
        assert!(matches!(
            CssTokenizer::tokenize(")")[0],
            CssToken::CloseParen
        ));
        assert!(matches!(
            CssTokenizer::tokenize("{")[0],
            CssToken::OpenCurly
        ));
        assert!(matches!(
            CssTokenizer::tokenize("}")[0],
            CssToken::CloseCurly
        ));
        assert!(matches!(
            CssTokenizer::tokenize("[")[0],
            CssToken::OpenSquare
        ));
        assert!(matches!(
            CssTokenizer::tokenize("]")[0],
            CssToken::CloseSquare
        ));
        assert!(matches!(CssTokenizer::tokenize(":")[0], CssToken::Colon));
        assert!(matches!(
            CssTokenizer::tokenize(";")[0],
            CssToken::Semicolon
        ));
        assert!(matches!(CssTokenizer::tokenize(",")[0], CssToken::Comma));
    }

    #[test]
    fn test_simple_identifiers() {
        let tokens = CssTokenizer::tokenize("div");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == "div"));
    }

    #[test]
    fn test_identifier_with_hyphen() {
        let tokens = CssTokenizer::tokenize("font-family");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == "font-family"));
    }

    #[test]
    fn test_identifier_starting_with_hyphen() {
        let tokens = CssTokenizer::tokenize("-webkit-transform");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == "-webkit-transform"));
    }

    #[test]
    fn test_identifier_with_double_hyphen() {
        let tokens = CssTokenizer::tokenize("--primary-color");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == "--primary-color"));
    }

    #[test]
    fn test_identifier_with_underscore() {
        let tokens = CssTokenizer::tokenize("_private_class");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == "_private_class"));
    }

    #[test]
    fn test_identifier_with_digits() {
        let tokens = CssTokenizer::tokenize("class123");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == "class123"));
    }

    #[test]
    fn test_identifier_unicode() {
        let tokens = CssTokenizer::tokenize("élément");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == "élément"));
    }

    #[test]
    fn test_identifier_with_escape() {
        let tokens = CssTokenizer::tokenize(r"\31 abc");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == "1abc"));
    }

    #[test]
    fn test_multiple_identifiers() {
        let tokens = CssTokenizer::tokenize("hello world");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == "hello"));
        assert!(matches!(tokens[1], CssToken::Whitespace));
        assert!(matches!(&tokens[2], CssToken::Ident(s) if s == "world"));
    }

    #[test]
    fn test_integer() {
        let tokens = CssTokenizer::tokenize("123");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Number(num) = &tokens[0] {
            assert_eq!(num.value, 123.0);
            assert_eq!(num.type_flag, NumberType::Integer);
            assert_eq!(num.int_value, Some(123));
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_negative_integer() {
        let tokens = CssTokenizer::tokenize("-42");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Number(num) = &tokens[0] {
            assert_eq!(num.value, -42.0);
            assert_eq!(num.type_flag, NumberType::Integer);
            assert_eq!(num.int_value, Some(-42));
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_positive_integer_with_sign() {
        let tokens = CssTokenizer::tokenize("+100");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Number(num) = &tokens[0] {
            assert_eq!(num.value, 100.0);
            assert_eq!(num.type_flag, NumberType::Integer);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_decimal_number() {
        let tokens = CssTokenizer::tokenize("3.14");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Number(num) = &tokens[0] {
            assert!((num.value - 3.14).abs() < 0.0001);
            assert_eq!(num.type_flag, NumberType::Number);
            assert_eq!(num.int_value, None);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_decimal_starting_with_dot() {
        let tokens = CssTokenizer::tokenize(".5");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Number(num) = &tokens[0] {
            assert!((num.value - 0.5).abs() < 0.0001);
            assert_eq!(num.type_flag, NumberType::Number);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_number_with_exponent() {
        let tokens = CssTokenizer::tokenize("1e10");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Number(num) = &tokens[0] {
            assert!((num.value - 1e10).abs() < 1e5);
            assert_eq!(num.type_flag, NumberType::Number);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_number_with_negative_exponent() {
        let tokens = CssTokenizer::tokenize("1e-5");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Number(num) = &tokens[0] {
            assert!((num.value - 1e-5).abs() < 1e-10);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_number_with_positive_exponent() {
        let tokens = CssTokenizer::tokenize("2E+3");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Number(num) = &tokens[0] {
            assert!((num.value - 2000.0).abs() < 0.0001);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_zero() {
        let tokens = CssTokenizer::tokenize("0");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Number(num) = &tokens[0] {
            assert_eq!(num.value, 0.0);
            assert_eq!(num.type_flag, NumberType::Integer);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_dimension_px() {
        let tokens = CssTokenizer::tokenize("100px");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Dimension { value, unit } = &tokens[0] {
            assert_eq!(value.value, 100.0);
            assert_eq!(unit, "px");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_dimension_em() {
        let tokens = CssTokenizer::tokenize("1.5em");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Dimension { value, unit } = &tokens[0] {
            assert!((value.value - 1.5).abs() < 0.0001);
            assert_eq!(unit, "em");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_dimension_rem() {
        let tokens = CssTokenizer::tokenize("2rem");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Dimension { value, unit } = &tokens[0] {
            assert_eq!(value.value, 2.0);
            assert_eq!(unit, "rem");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_dimension_deg() {
        let tokens = CssTokenizer::tokenize("45deg");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Dimension { value, unit } = &tokens[0] {
            assert_eq!(value.value, 45.0);
            assert_eq!(unit, "deg");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_dimension_ms() {
        let tokens = CssTokenizer::tokenize("300ms");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Dimension { value, unit } = &tokens[0] {
            assert_eq!(value.value, 300.0);
            assert_eq!(unit, "ms");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_dimension_s() {
        let tokens = CssTokenizer::tokenize("0.5s");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Dimension { value, unit } = &tokens[0] {
            assert!((value.value - 0.5).abs() < 0.0001);
            assert_eq!(unit, "s");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_dimension_vw() {
        let tokens = CssTokenizer::tokenize("100vw");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Dimension { value, unit } = &tokens[0] {
            assert_eq!(value.value, 100.0);
            assert_eq!(unit, "vw");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_dimension_negative() {
        let tokens = CssTokenizer::tokenize("-10px");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Dimension { value, unit } = &tokens[0] {
            assert_eq!(value.value, -10.0);
            assert_eq!(unit, "px");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_percentage() {
        let tokens = CssTokenizer::tokenize("50%");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Percentage(num) = &tokens[0] {
            assert_eq!(num.value, 50.0);
        } else {
            panic!("Expected Percentage token");
        }
    }

    #[test]
    fn test_percentage_decimal() {
        let tokens = CssTokenizer::tokenize("33.33%");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Percentage(num) = &tokens[0] {
            assert!((num.value - 33.33).abs() < 0.0001);
        } else {
            panic!("Expected Percentage token");
        }
    }

    #[test]
    fn test_percentage_negative() {
        let tokens = CssTokenizer::tokenize("-25%");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Percentage(num) = &tokens[0] {
            assert_eq!(num.value, -25.0);
        } else {
            panic!("Expected Percentage token");
        }
    }

    #[test]
    fn test_double_quoted_string() {
        let tokens = CssTokenizer::tokenize(r#""hello world""#);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::String(s) if s == "hello world"));
    }

    #[test]
    fn test_single_quoted_string() {
        let tokens = CssTokenizer::tokenize("'hello world'");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::String(s) if s == "hello world"));
    }

    #[test]
    fn test_empty_string() {
        let tokens = CssTokenizer::tokenize(r#""""#);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::String(s) if s.is_empty()));
    }

    #[test]
    fn test_string_with_escaped_quote() {
        let tokens = CssTokenizer::tokenize(r#""hello \"world\"""#);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::String(s) if s == r#"hello "world""#));
    }

    #[test]
    fn test_string_with_escaped_newline() {
        let tokens = CssTokenizer::tokenize("\"hello\\\nworld\"");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::String(s) if s == "helloworld"));
    }

    #[test]
    fn test_string_with_hex_escape() {
        let tokens = CssTokenizer::tokenize(r#""\41 BC""#);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::String(s) if s == "ABC"));
    }

    #[test]
    fn test_string_with_unicode_escape() {
        let tokens = CssTokenizer::tokenize(r#""\1F600""#);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::String(s) if s == "😀"));
    }

    #[test]
    fn test_hash_id() {
        let tokens = CssTokenizer::tokenize("#header");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Hash { value, type_flag } = &tokens[0] {
            assert_eq!(value, "header");
            assert_eq!(*type_flag, HashType::Id);
        } else {
            panic!("Expected Hash token");
        }
    }

    #[test]
    fn test_hash_color_hex3() {
        let tokens = CssTokenizer::tokenize("#fff");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Hash { value, type_flag } = &tokens[0] {
            assert_eq!(value, "fff");
            assert_eq!(*type_flag, HashType::Id);
        } else {
            panic!("Expected Hash token");
        }
    }

    #[test]
    fn test_hash_color_hex6() {
        let tokens = CssTokenizer::tokenize("#ff00ff");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Hash { value, type_flag } = &tokens[0] {
            assert_eq!(value, "ff00ff");
            assert_eq!(*type_flag, HashType::Id);
        } else {
            panic!("Expected Hash token");
        }
    }

    #[test]
    fn test_hash_numeric_start() {
        let tokens = CssTokenizer::tokenize("#123");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Hash { value, type_flag } = &tokens[0] {
            assert_eq!(value, "123");
            assert_eq!(*type_flag, HashType::Unrestricted);
        } else {
            panic!("Expected Hash token");
        }
    }

    #[test]
    fn test_hash_alphanumeric() {
        let tokens = CssTokenizer::tokenize("#abc123");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Hash { value, type_flag } = &tokens[0] {
            assert_eq!(value, "abc123");
            assert_eq!(*type_flag, HashType::Id);
        } else {
            panic!("Expected Hash token");
        }
    }

    #[test]
    fn test_function_rgb() {
        let tokens = CssTokenizer::tokenize("rgb(");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Function(s) if s == "rgb"));
    }

    #[test]
    fn test_function_rgba() {
        let tokens = CssTokenizer::tokenize("rgba(");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Function(s) if s == "rgba"));
    }

    #[test]
    fn test_function_calc() {
        let tokens = CssTokenizer::tokenize("calc(");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Function(s) if s == "calc"));
    }

    #[test]
    fn test_function_var() {
        let tokens = CssTokenizer::tokenize("var(");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Function(s) if s == "var"));
    }

    #[test]
    fn test_function_complete_call() {
        let tokens = CssTokenizer::tokenize("rgb(255, 0, 0)");
        assert_eq!(tokens.len(), 9);
        assert!(matches!(&tokens[0], CssToken::Function(s) if s == "rgb"));
        assert!(matches!(&tokens[1], CssToken::Number(_)));
        assert!(matches!(tokens[2], CssToken::Comma));
        assert!(matches!(tokens[3], CssToken::Whitespace));
        assert!(matches!(&tokens[4], CssToken::Number(_)));
        assert!(matches!(tokens[5], CssToken::Comma));
        assert!(matches!(tokens[6], CssToken::Whitespace));
        assert!(matches!(&tokens[7], CssToken::Number(_)));
        assert!(matches!(tokens[8], CssToken::CloseParen));
    }

    #[test]
    fn test_url_unquoted() {
        let tokens = CssTokenizer::tokenize("url(image.png)");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Url(s) if s == "image.png"));
    }

    #[test]
    fn test_url_with_path() {
        let tokens = CssTokenizer::tokenize("url(/path/to/image.png)");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Url(s) if s == "/path/to/image.png"));
    }

    #[test]
    fn test_url_quoted_double() {
        let tokens = CssTokenizer::tokenize("url(\"image.png\")");
        assert!(tokens.len() >= 2);
        assert!(matches!(&tokens[0], CssToken::Function(s) if s == "url"));
        assert!(matches!(&tokens[1], CssToken::String(s) if s == "image.png"));
    }

    #[test]
    fn test_url_quoted_single() {
        let tokens = CssTokenizer::tokenize("url('image.png')");
        assert!(tokens.len() >= 2);
        assert!(matches!(&tokens[0], CssToken::Function(s) if s == "url"));
        assert!(matches!(&tokens[1], CssToken::String(s) if s == "image.png"));
    }

    #[test]
    fn test_url_with_whitespace() {
        let tokens = CssTokenizer::tokenize("url(  image.png  )");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Url(s) if s == "image.png"));
    }

    #[test]
    fn test_at_keyword_media() {
        let tokens = CssTokenizer::tokenize("@media");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::AtKeyword(s) if s == "media"));
    }

    #[test]
    fn test_at_keyword_import() {
        let tokens = CssTokenizer::tokenize("@import");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::AtKeyword(s) if s == "import"));
    }

    #[test]
    fn test_at_keyword_keyframes() {
        let tokens = CssTokenizer::tokenize("@keyframes");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::AtKeyword(s) if s == "keyframes"));
    }

    #[test]
    fn test_at_keyword_font_face() {
        let tokens = CssTokenizer::tokenize("@font-face");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::AtKeyword(s) if s == "font-face"));
    }

    #[test]
    fn test_at_keyword_supports() {
        let tokens = CssTokenizer::tokenize("@supports");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::AtKeyword(s) if s == "supports"));
    }

    #[test]
    fn test_at_keyword_charset() {
        let tokens = CssTokenizer::tokenize("@charset");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::AtKeyword(s) if s == "charset"));
    }

    #[test]
    fn test_at_not_keyword() {
        let tokens = CssTokenizer::tokenize("@ ");
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], CssToken::Delim('@')));
        assert!(matches!(tokens[1], CssToken::Whitespace));
    }

    #[test]
    fn test_cdo() {
        let tokens = CssTokenizer::tokenize("<!--");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], CssToken::Cdo));
    }

    #[test]
    fn test_cdc() {
        let tokens = CssTokenizer::tokenize("-->");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], CssToken::Cdc));
    }

    #[test]
    fn test_cdo_cdc_in_context() {
        let tokens = CssTokenizer::tokenize("<!-- div { } -->");
        assert!(tokens.len() >= 5);
        assert!(matches!(tokens[0], CssToken::Cdo));
        assert!(matches!(tokens[tokens.len() - 1], CssToken::Cdc));
    }

    #[test]
    fn test_delim_plus() {
        let tokens = CssTokenizer::tokenize("+ ");
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], CssToken::Delim('+')));
    }

    #[test]
    fn test_delim_dot_not_number() {
        let tokens = CssTokenizer::tokenize(".class");
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], CssToken::Delim('.')));
        assert!(matches!(&tokens[1], CssToken::Ident(s) if s == "class"));
    }

    #[test]
    fn test_delim_asterisk() {
        let tokens = CssTokenizer::tokenize("*");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], CssToken::Delim('*')));
    }

    #[test]
    fn test_delim_greater_than() {
        let tokens = CssTokenizer::tokenize(">");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], CssToken::Delim('>')));
    }

    #[test]
    fn test_delim_tilde() {
        let tokens = CssTokenizer::tokenize("~");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], CssToken::Delim('~')));
    }

    #[test]
    fn test_delim_pipe() {
        let tokens = CssTokenizer::tokenize("|");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], CssToken::Delim('|')));
    }

    #[test]
    fn test_delim_equals() {
        let tokens = CssTokenizer::tokenize("=");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], CssToken::Delim('=')));
    }

    #[test]
    fn test_delim_caret() {
        let tokens = CssTokenizer::tokenize("^");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], CssToken::Delim('^')));
    }

    #[test]
    fn test_delim_dollar() {
        let tokens = CssTokenizer::tokenize("$");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], CssToken::Delim('$')));
    }

    #[test]
    fn test_simple_comment() {
        let tokens = CssTokenizer::tokenize("/* comment */");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_comment_before_rule() {
        let tokens = CssTokenizer::tokenize("/* comment */ div");
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], CssToken::Whitespace));
        assert!(matches!(&tokens[1], CssToken::Ident(s) if s == "div"));
    }

    #[test]
    fn test_multiline_comment() {
        let tokens = CssTokenizer::tokenize("/* line 1\n   line 2\n   line 3 */");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_nested_comment_markers() {
        let tokens = CssTokenizer::tokenize("/* outer /* inner */ */");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], CssToken::Whitespace));
        assert!(matches!(tokens[1], CssToken::Delim('*')));
        assert!(matches!(tokens[2], CssToken::Delim('/')));
    }

    #[test]
    fn test_comment_with_special_chars() {
        let tokens = CssTokenizer::tokenize("/* <>&\"' */");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_escape_single_char() {
        let tokens = CssTokenizer::tokenize(r"\.class");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == ".class"));
    }

    #[test]
    fn test_escape_hex_1_digit() {
        let tokens = CssTokenizer::tokenize(r"\A");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == "\n"));
    }

    #[test]
    fn test_escape_hex_6_digits() {
        let tokens = CssTokenizer::tokenize(r"\000041BC");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == "ABC"));
    }

    #[test]
    fn test_escape_followed_by_space() {
        let tokens = CssTokenizer::tokenize(r"\41 B");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == "AB"));
    }

    #[test]
    fn test_crlf_to_lf() {
        let tokens = CssTokenizer::tokenize("a\r\nb");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == "a"));
        assert!(matches!(tokens[1], CssToken::Whitespace));
        assert!(matches!(&tokens[2], CssToken::Ident(s) if s == "b"));
    }

    #[test]
    fn test_cr_to_lf() {
        let tokens = CssTokenizer::tokenize("a\rb");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[1], CssToken::Whitespace));
    }

    #[test]
    fn test_ff_to_lf() {
        let tokens = CssTokenizer::tokenize("a\x0Cb");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[1], CssToken::Whitespace));
    }

    #[test]
    fn test_null_to_replacement() {
        let tokens = CssTokenizer::tokenize("a\0b");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == "a\u{FFFD}b"));
    }

    #[test]
    fn test_simple_rule() {
        let tokens = CssTokenizer::tokenize("div { color: red; }");
        assert!(tokens.len() >= 9);
        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == "div"));
    }

    #[test]
    fn test_class_selector() {
        let tokens = CssTokenizer::tokenize(".container");
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], CssToken::Delim('.')));
        assert!(matches!(&tokens[1], CssToken::Ident(s) if s == "container"));
    }

    #[test]
    fn test_id_selector() {
        let tokens = CssTokenizer::tokenize("#main");
        assert_eq!(tokens.len(), 1);
        assert!(
            matches!(&tokens[0], CssToken::Hash { value, type_flag: HashType::Id } if value == "main")
        );
    }

    #[test]
    fn test_attribute_selector() {
        let tokens = CssTokenizer::tokenize("[data-attr=\"value\"]");
        assert!(tokens.len() >= 5);
        assert!(matches!(tokens[0], CssToken::OpenSquare));
        assert!(matches!(&tokens[1], CssToken::Ident(s) if s == "data-attr"));
    }

    #[test]
    fn test_pseudo_class() {
        let tokens = CssTokenizer::tokenize(":hover");
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], CssToken::Colon));
        assert!(matches!(&tokens[1], CssToken::Ident(s) if s == "hover"));
    }

    #[test]
    fn test_pseudo_element() {
        let tokens = CssTokenizer::tokenize("::before");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], CssToken::Colon));
        assert!(matches!(tokens[1], CssToken::Colon));
        assert!(matches!(&tokens[2], CssToken::Ident(s) if s == "before"));
    }

    #[test]
    fn test_media_query() {
        let tokens = CssTokenizer::tokenize("@media screen and (min-width: 768px)");
        assert!(tokens.len() >= 8);
        assert!(matches!(&tokens[0], CssToken::AtKeyword(s) if s == "media"));
    }

    #[test]
    fn test_calc_expression() {
        let tokens = CssTokenizer::tokenize("calc(100% - 20px)");
        assert!(tokens.len() >= 6);
        assert!(matches!(&tokens[0], CssToken::Function(s) if s == "calc"));
    }

    #[test]
    fn test_var_function() {
        let tokens = CssTokenizer::tokenize("var(--primary-color)");
        assert!(tokens.len() >= 3);
        assert!(matches!(&tokens[0], CssToken::Function(s) if s == "var"));
        assert!(matches!(&tokens[1], CssToken::Ident(s) if s == "--primary-color"));
    }

    #[test]
    fn test_gradient() {
        let tokens = CssTokenizer::tokenize("linear-gradient(to right, red, blue)");
        assert!(tokens.len() >= 8);
        assert!(matches!(&tokens[0], CssToken::Function(s) if s == "linear-gradient"));
    }

    #[test]
    fn test_important() {
        let tokens = CssTokenizer::tokenize("!important");
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], CssToken::Delim('!')));
        assert!(matches!(&tokens[1], CssToken::Ident(s) if s == "important"));
    }

    #[test]
    fn test_token_display_ident() {
        let token = CssToken::Ident("hello".to_string());
        assert_eq!(format!("{}", token), "hello");
    }

    #[test]
    fn test_token_display_function() {
        let token = CssToken::Function("rgb".to_string());
        assert_eq!(format!("{}", token), "rgb(");
    }

    #[test]
    fn test_token_display_at_keyword() {
        let token = CssToken::AtKeyword("media".to_string());
        assert_eq!(format!("{}", token), "@media");
    }

    #[test]
    fn test_token_display_hash() {
        let token = CssToken::Hash {
            value: "fff".to_string(),
            type_flag: HashType::Id,
        };
        assert_eq!(format!("{}", token), "#fff");
    }

    #[test]
    fn test_token_display_string() {
        let token = CssToken::String("hello".to_string());
        assert_eq!(format!("{}", token), "\"hello\"");
    }

    #[test]
    fn test_token_display_url() {
        let token = CssToken::Url("image.png".to_string());
        assert_eq!(format!("{}", token), "url(image.png)");
    }

    #[test]
    fn test_token_display_number() {
        let token = CssToken::Number(NumericValue::new(
            42.0,
            "42".to_string(),
            NumberType::Integer,
        ));
        assert_eq!(format!("{}", token), "42");
    }

    #[test]
    fn test_token_display_percentage() {
        let token = CssToken::Percentage(NumericValue::new(
            50.0,
            "50".to_string(),
            NumberType::Integer,
        ));
        assert_eq!(format!("{}", token), "50%");
    }

    #[test]
    fn test_token_display_dimension() {
        let token = CssToken::Dimension {
            value: NumericValue::new(100.0, "100".to_string(), NumberType::Integer),
            unit: "px".to_string(),
        };
        assert_eq!(format!("{}", token), "100px");
    }

    #[test]
    fn test_token_display_delim() {
        let token = CssToken::Delim('*');
        assert_eq!(format!("{}", token), "*");
    }

    #[test]
    fn test_token_display_brackets() {
        assert_eq!(format!("{}", CssToken::OpenParen), "(");
        assert_eq!(format!("{}", CssToken::CloseParen), ")");
        assert_eq!(format!("{}", CssToken::OpenCurly), "{");
        assert_eq!(format!("{}", CssToken::CloseCurly), "}");
        assert_eq!(format!("{}", CssToken::OpenSquare), "[");
        assert_eq!(format!("{}", CssToken::CloseSquare), "]");
    }

    #[test]
    fn test_token_display_punctuation() {
        assert_eq!(format!("{}", CssToken::Colon), ":");
        assert_eq!(format!("{}", CssToken::Semicolon), ";");
        assert_eq!(format!("{}", CssToken::Comma), ",");
    }

    #[test]
    fn test_token_display_whitespace() {
        assert_eq!(format!("{}", CssToken::Whitespace), " ");
    }

    #[test]
    fn test_token_display_cdo_cdc() {
        assert_eq!(format!("{}", CssToken::Cdo), "<!--");
        assert_eq!(format!("{}", CssToken::Cdc), "-->");
    }

    #[test]
    fn test_token_display_eof() {
        assert_eq!(format!("{}", CssToken::Eof), "");
    }

    #[test]
    fn test_unterminated_string_returns_bad_string() {
        let tokens = CssTokenizer::tokenize("\"unterminated");
        assert!(!tokens.is_empty());
        assert!(
            matches!(&tokens[0], CssToken::String(_)) || matches!(&tokens[0], CssToken::BadString)
        );
    }

    #[test]
    fn test_string_with_newline_returns_bad_string() {
        let tokens = CssTokenizer::tokenize("\"bad\nstring\"");
        assert!(!tokens.is_empty());
        assert!(matches!(tokens[0], CssToken::BadString));
    }

    #[test]
    fn test_invalid_escape_at_top_level() {
        let tokens = CssTokenizer::tokenize("\\\n");
        assert!(!tokens.is_empty());
        assert!(matches!(tokens[0], CssToken::Delim('\\')));
    }

    #[test]
    fn test_multiple_consecutive_numbers() {
        let tokens = CssTokenizer::tokenize("1 2 3");
        assert_eq!(tokens.len(), 5);
        assert!(matches!(&tokens[0], CssToken::Number(n) if n.value == 1.0));
        assert!(matches!(&tokens[2], CssToken::Number(n) if n.value == 2.0));
        assert!(matches!(&tokens[4], CssToken::Number(n) if n.value == 3.0));
    }

    #[test]
    fn test_number_immediately_followed_by_ident() {
        let tokens = CssTokenizer::tokenize("10abc");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Dimension { unit, .. } if unit == "abc"));
    }

    #[test]
    fn test_negative_number_as_dimension() {
        let tokens = CssTokenizer::tokenize("-5em");
        assert_eq!(tokens.len(), 1);
        if let CssToken::Dimension { value, unit } = &tokens[0] {
            assert_eq!(value.value, -5.0);
            assert_eq!(unit, "em");
        } else {
            panic!("Expected Dimension token");
        }
    }

    #[test]
    fn test_very_long_identifier() {
        let long_ident = "a".repeat(1000);
        let tokens = CssTokenizer::tokenize(&long_ident);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Ident(s) if s == &long_ident));
    }

    #[test]
    fn test_very_large_number() {
        let tokens = CssTokenizer::tokenize("999999999999999999999");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Number(_)));
    }

    #[test]
    fn test_scientific_notation_edge_cases() {
        let tokens = CssTokenizer::tokenize("1eabc");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], CssToken::Dimension { unit, .. } if unit == "eabc"));
    }
}
