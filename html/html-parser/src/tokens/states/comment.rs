use html_syntax::token::{Token, TokenKind};
use std::collections::HashMap;

use crate::tokens::{state::ParserState, tokenizer::HtmlTokenizer};

/// Handles the bogus comment state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is '>', the tokenizer transitions back to the `ParserState::Data` state.
pub fn handle_bogus_comment_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    if ch == '>' {
        tokenizer.state = ParserState::Data;
    }
}

/// Handles the comment start state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is '-', a new comment token is created and the tokenizer transitions to the `ParserState::Comment` state.
/// - For any other character, the tokenizer transitions to the `ParserState::BogusComment` state.
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

/// Handles the comment state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is '-', the tokenizer transitions to the `ParserState::CommentEnd` state.
/// - For any other character, it appends the character to the current comment token's data.
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

/// Handles the comment end state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is '>', the current comment token is emitted and the tokenizer transitions back to the `ParserState::Data` state.
/// - If the character is '-', it remains in the `ParserState::CommentEnd` state.
/// - For any other character, it appends a '-' and the character to the current comment token's data and transitions back to the `ParserState::Comment` state.
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
