//! CSS Tokenizer implementation following CSS Syntax Module Level 3
//! <https://www.w3.org/TR/css-syntax-3/#tokenization>

use crate::{consumers::token::consume_token, tokens::CssToken};

/// Input stream for the tokenizer
pub struct InputStream {
    /// Characters of the input
    chars: Vec<char>,

    /// Current position in the input
    pos: usize,

    /// Current character
    pub current: Option<char>,
}

impl InputStream {
    /// Create a new input stream from the given string
    pub fn new(input: &str) -> Self {
        InputStream {
            chars: input.chars().collect(),
            pos: 0,
            current: None,
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
            self.current = Some(self.chars[self.pos]);
            self.pos += 1;
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
        }
    }
}

/// CSS Tokenizer following CSS Syntax Module Level 3
pub struct CssTokenizer {
    /// Input stream for the tokenizer
    pub stream: InputStream,
}

impl CssTokenizer {
    /// Create a new tokenizer from the given input string
    ///
    /// # Arguments
    /// * `input` - The input CSS string to tokenize
    fn new(input: &str) -> Self {
        CssTokenizer {
            stream: InputStream::new(input),
        }
    }

    /// Tokenize the input and return a vector of tokens
    fn collect(&mut self) -> Vec<CssToken> {
        let mut tokens = Vec::new();

        loop {
            let token = consume_token(self);

            if matches!(token, CssToken::Eof) {
                break;
            }

            tokens.push(token);
        }

        tokens
    }

    /// Tokenize the given input string and return a vector of tokens
    ///
    /// # Arguments
    /// * `input` - The input CSS string to tokenize
    pub fn tokenize(input: &str) -> Vec<CssToken> {
        let mut tokenizer = CssTokenizer::new(input);
        tokenizer.collect()
    }
}
