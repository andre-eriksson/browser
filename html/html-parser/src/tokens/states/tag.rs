use std::collections::HashMap;

use html_syntax::token::{Token, TokenKind};

use crate::tokens::{
    state::ParserState,
    tokenizer::{HtmlTokenizer, TokenizerState},
};

/// Handles the tag open state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is '!', it transitions to the `ParserState::StartDeclaration` state.
/// - If the character is '?', it creates a new XML declaration token and transitions to the `ParserState::XmlDeclaration` state.
/// - If the character is '/', it creates a new end tag token and transitions to the `ParserState::EndTagOpen` state.
/// - If the character is alphabetic, it creates a new start tag token with the character and transitions to the `ParserState::TagName` state.
/// - For any other character, it transitions back to the `ParserState::Data` state.
pub fn handle_tag_open_state(state: &mut TokenizerState, ch: char) {
    match ch {
        '!' => {
            state.state = ParserState::StartDeclaration;
        }
        '?' => {
            state.current_token = Some(Token {
                kind: TokenKind::XmlDeclaration,
                attributes: HashMap::new(),
                data: ch.to_string(),
            });
            state.state = ParserState::XmlDeclaration;
        }
        '/' => {
            state.current_token = Some(Token {
                kind: TokenKind::EndTag,
                attributes: HashMap::new(),
                data: String::new(),
            });
            state.state = ParserState::EndTagOpen;
        }
        ch if ch.is_alphabetic() => {
            state.current_token = Some(Token {
                kind: TokenKind::StartTag,
                attributes: HashMap::new(),
                data: ch.to_string(),
            });
            state.state = ParserState::TagName;
        }
        _ => {
            state.state = ParserState::Data;
        }
    }
}

/// Handles the end tag open state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
/// * `tokens` - A mutable reference to the vector of tokens to which new tokens will be emitted.
///
/// # Behavior
/// - If the character is '>', it emits the current end tag token and transitions to the `ParserState::Data` state.
/// - If the character is alphabetic, it appends the character to the current end tag token's data and transitions to the `ParserState::TagName` state.
/// - For any other character, it transitions back to the `ParserState::Data` state.
pub fn handle_end_tag_open_state(state: &mut TokenizerState, ch: char, tokens: &mut Vec<Token>) {
    match ch {
        '>' => {
            if let Some(token) = state.current_token.take() {
                HtmlTokenizer::emit_token(tokens, token);
            }

            state.state = ParserState::Data;
        }
        ch if ch.is_alphabetic() => {
            if let Some(token) = state.current_token.as_mut() {
                token.data.push(ch);
            } else {
                state.current_token = Some(Token {
                    kind: TokenKind::EndTag,
                    attributes: HashMap::new(),
                    data: ch.to_string(),
                });
            }
            state.state = ParserState::TagName;
        }
        _ => {
            state.state = ParserState::Data;
        }
    }
}

/// Handles the self-closing tag start state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
/// * `tokens` - A mutable reference to the vector of tokens to which new tokens will be emitted.
///
/// # Behavior
/// - If the character is '>', it finalizes the current token, emits it, and transitions to the `ParserState::Data` state.
/// - If the character is whitespace, it ignores it.
/// - For any other character, it transitions to the `ParserState::BeforeAttributeName` state.
pub fn handle_self_closing_tag_start_state(
    state: &mut TokenizerState,
    ch: char,
    tokens: &mut Vec<Token>,
) {
    match ch {
        '>' => {
            if let Some(mut token) = state.current_token.take() {
                if !state.current_attribute_name.is_empty() {
                    token.attributes.insert(
                        state.current_attribute_name.clone(),
                        state.current_attribute_value.clone(),
                    );

                    state.current_attribute_name.clear();
                    state.current_attribute_value.clear();
                }

                HtmlTokenizer::emit_token(tokens, token);
            }
            state.state = ParserState::Data;
        }
        ch if ch.is_whitespace() => {}
        _ => {
            state.state = ParserState::BeforeAttributeName;
        }
    }
}

/// Handles the tag name state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
/// * `tokens` - A mutable reference to the vector of tokens to which new tokens will be emitted.
///
/// # Behavior
/// - If the character is '>', it finalizes the current token, emits it, and transitions to the appropriate state based on the tag name.
/// - If the character is '/', it transitions to the `ParserState::SelfClosingTagStart` state.
/// - If the character is whitespace, it transitions to the `ParserState::BeforeAttributeName` state.
/// - For any other character, it appends the character to the current token's data.
pub fn handle_tag_name_state(state: &mut TokenizerState, ch: char, tokens: &mut Vec<Token>) {
    match ch {
        '>' => {
            if let Some(token) = state.current_token.take() {
                if token.data == "script" {
                    state.state = ParserState::ScriptData;
                } else {
                    if token.data == "pre" {
                        if token.kind == TokenKind::StartTag {
                            state.context.inside_preformatted = true;
                        } else if token.kind == TokenKind::EndTag {
                            state.context.inside_preformatted = false;
                        }
                    }

                    state.state = ParserState::Data;
                }

                HtmlTokenizer::emit_token(tokens, token);
            }
        }
        '/' => {
            state.state = ParserState::SelfClosingTagStart;
        }
        ch if ch.is_whitespace() => {
            state.state = ParserState::BeforeAttributeName;
        }
        _ => {
            if let Some(token) = state.current_token.as_mut() {
                token.data.push(ch);
            } else {
                state.current_token = Some(Token {
                    kind: TokenKind::StartTag,
                    attributes: HashMap::new(),
                    data: ch.to_string(),
                });
            }
        }
    }
}
