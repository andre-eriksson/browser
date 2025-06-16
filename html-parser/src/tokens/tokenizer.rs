use logos::Logos;

use crate::tokens::token::Token;

pub struct Tokenizer {
    input: String,
}

impl Tokenizer {
    pub fn new(input: String) -> Self {
        Tokenizer { input }
    }

    pub fn tokenize(&self) -> Vec<(Token, &str)> {
        let mut lexer = Token::lexer(self.input.as_str());

        let mut tokens = Vec::new();
        while let Some(token_result) = lexer.next() {
            if let Ok(token) = token_result {
                tokens.push((token, lexer.slice()));
            }
        }

        tokens
    }
}
