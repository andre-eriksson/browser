use std::collections::HashMap;

use crate::tokens::{
    state::{ParserState, Token, TokenKind},
    tokenizer::HtmlTokenizer,
};

pub fn handle_bogus_comment_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '>' => {
            tokenizer.state = ParserState::Data;
        }
        _ => {}
    }
}

pub fn handle_comment_start_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '-' => {
            tokenizer.current_token = Some(Token {
                kind: TokenKind::Comment,
                attributes: HashMap::new(),
                data: String::new(),
            });

            tokenizer.state = ParserState::Comment;
        }
        _ => {
            tokenizer.state = ParserState::BogusComment;
        }
    }
}

pub fn handle_comment_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '-' => {
            tokenizer.state = ParserState::CommentEnd;
        }
        _ => {
            if let Some(token) = tokenizer.current_token.as_mut() {
                token.data.push(ch);
            } else {
                tokenizer.current_token = Some(Token {
                    kind: TokenKind::Comment,
                    attributes: HashMap::new(),
                    data: ch.to_string(),
                });
            }
        }
    }
}

pub fn handle_comment_end_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '>' => {
            if let Some(token) = tokenizer.current_token.take() {
                tokenizer.emit_token(token);
            }
            tokenizer.state = ParserState::Data;
        }
        '-' => {}
        _ => {
            if let Some(token) = tokenizer.current_token.as_mut() {
                token.data.push('-');
                token.data.push(ch);
            } else {
                tokenizer.current_token = Some(Token {
                    kind: TokenKind::Comment,
                    attributes: HashMap::new(),
                    data: format!("-{}", ch),
                });
            }
            tokenizer.state = ParserState::Comment;
        }
    }
}
