use html_syntax::token::{Token, TokenKind};
use std::collections::HashMap;

use crate::{
    state::ParserState,
    tokenizer::{HtmlTokenizer, TokenizerState},
};

/// Handles the bogus comment state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is '>', the tokenizer transitions back to the `ParserState::Data` state.
pub fn handle_bogus_comment_state(state: &mut TokenizerState, ch: char) {
    if ch == '>' {
        state.state = ParserState::Data;
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
pub fn handle_comment_start_state(state: &mut TokenizerState, ch: char) {
    match ch {
        '-' => {
            state.current_token = Some(Token {
                kind: TokenKind::Comment,
                attributes: HashMap::new(),
                data: String::new(),
            });

            state.state = ParserState::Comment;
        }
        _ => {
            state.state = ParserState::BogusComment;
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
pub fn handle_comment_state(state: &mut TokenizerState, ch: char) {
    match ch {
        '-' => {
            state.state = ParserState::CommentEnd;
        }
        _ => {
            if let Some(token) = state.current_token.as_mut() {
                token.data.push(ch);
            } else {
                state.current_token = Some(Token {
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
/// * `tokens` - A mutable reference to the vector of tokens to which new tokens will be emitted.
///
/// # Behavior
/// - If the character is '>', the current comment token is emitted and the tokenizer transitions back to the `ParserState::Data` state.
/// - If the character is '-', it remains in the `ParserState::CommentEnd` state.
/// - For any other character, it appends a '-' and the character to the current comment token's data and transitions back to the `ParserState::Comment` state.
pub fn handle_comment_end_state(state: &mut TokenizerState, ch: char, tokens: &mut Vec<Token>) {
    match ch {
        '>' => {
            if let Some(token) = state.current_token.take() {
                HtmlTokenizer::emit_token(tokens, token);
            }
            state.state = ParserState::Data;
        }
        '-' => {}
        _ => {
            if let Some(token) = state.current_token.as_mut() {
                token.data.push('-');
                token.data.push(ch);
            } else {
                state.current_token = Some(Token {
                    kind: TokenKind::Comment,
                    attributes: HashMap::new(),
                    data: format!("-{}", ch),
                });
            }
            state.state = ParserState::Comment;
        }
    }
}
