use std::collections::HashMap;

use html_syntax::token::{Token, TokenKind};

use crate::tokens::{state::ParserState, tokenizer::HtmlTokenizer};

/// Handles the start declaration state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is '-', the tokenizer transitions to the `ParserState::CommentStart` state.
/// - If the character is 'd' or 'D', a new doctype declaration token is created and the tokenizer transitions to the `ParserState::DoctypeDeclaration` state.
/// - If the character is whitespace, the tokenizer remains in the current state.
/// - For any other character, the tokenizer transitions to the `ParserState::BogusComment` state.
pub fn handle_start_declaration_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '-' => {
            tokenizer.state = ParserState::CommentStart;
        }
        'd' | 'D' => {
            tokenizer.current_token = Some(Token {
                kind: TokenKind::DoctypeDeclaration,
                attributes: HashMap::new(),
                data: ch.to_string(),
            });
            tokenizer.state = ParserState::DoctypeDeclaration;
        }
        ch if ch.is_whitespace() => {}
        _ => {
            tokenizer.state = ParserState::BogusComment;
        }
    }
}

/// Handles the XML declaration state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is '?', it appends it to the current XML declaration token's data.
/// - If the character is '>' and the current token's data ends with '?', it emits the token and transitions to the `ParserState::Data` state.
/// - For any other character, it appends it to the current XML declaration token's data.
pub fn handle_xml_declaration_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '?' => {
            if let Some(token) = tokenizer.current_token.as_mut() {
                token.data.push(ch);
            } else {
                tokenizer.current_token = Some(Token {
                    kind: TokenKind::XmlDeclaration,
                    attributes: HashMap::new(),
                    data: ch.to_string(),
                });
            }
        }
        '>' => {
            if let Some(token) = &tokenizer.current_token
                && token.data.ends_with('?')
            {
                if let Some(token) = tokenizer.current_token.take() {
                    tokenizer.emit_token(token);
                }
                tokenizer.state = ParserState::Data;
                return;
            }
            if let Some(token) = tokenizer.current_token.as_mut() {
                token.data.push(ch);
            } else {
                tokenizer.current_token = Some(Token {
                    kind: TokenKind::XmlDeclaration,
                    attributes: HashMap::new(),
                    data: ch.to_string(),
                });
            }
        }
        _ => {
            if let Some(token) = tokenizer.current_token.as_mut() {
                token.data.push(ch);
            } else {
                tokenizer.current_token = Some(Token {
                    kind: TokenKind::XmlDeclaration,
                    attributes: HashMap::new(),
                    data: ch.to_string(),
                });
            }
        }
    }
}

/// Handles the doctype declaration state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is '>', the current doctype declaration token is emitted and the tokenizer transitions back to the `ParserState::Data` state.
/// - For any other character, it appends the character to the current doctype declaration token's data.
pub fn handle_doctype_declaration_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '>' => {
            if let Some(token) = tokenizer.current_token.take() {
                tokenizer.emit_token(token);
            }
            tokenizer.state = ParserState::Data;
        }
        _ => {
            if let Some(token) = tokenizer.current_token.as_mut() {
                token.data.push(ch);
            } else {
                tokenizer.current_token = Some(Token {
                    kind: TokenKind::DoctypeDeclaration,
                    attributes: HashMap::new(),
                    data: ch.to_string(),
                });
            }
        }
    }
}
