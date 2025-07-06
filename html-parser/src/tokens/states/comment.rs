use std::collections::HashMap;

use crate::tokens::{
    state::{ParserState, Token, TokenKind},
    tokenizer::HtmlTokenizer,
};

pub fn handle_bogus_comment_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '>' => {
            // End of bogus comment
            tokenizer.state = ParserState::Data; // Return to Data state
        }
        _ => {
            // Ignore characters in bogus comment
        }
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

            tokenizer.state = ParserState::Comment; // Transition to Comment state
        }
        _ => {
            tokenizer.state = ParserState::BogusComment; // Transition to BogusComment state
        }
    }
}

pub fn handle_comment_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '-' => {
            tokenizer.state = ParserState::CommentEnd; // Transition to CommentEnd state
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
            // End of comment
            if let Some(token) = tokenizer.current_token.take() {
                tokenizer.emit_token(token);
            }
            tokenizer.state = ParserState::Data; // Return to Data state
        }
        '-' => {
            // Ignore consecutive dashes in comments
        }
        _ => {
            // Handle invalid characters after comment end
            if let Some(token) = tokenizer.current_token.as_mut() {
                token.data.push('-'); // Add the dash back to the comment data
                token.data.push(ch);
            } else {
                tokenizer.current_token = Some(Token {
                    kind: TokenKind::Comment,
                    attributes: HashMap::new(),
                    data: format!("-{}", ch),
                });
            }
            tokenizer.state = ParserState::Comment; // Return to Comment state
        }
    }
}
