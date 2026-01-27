use std::collections::HashMap;

use crate::{
    Token, TokenKind,
    state::TokenState,
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
            state.state = TokenState::StartDeclaration;
        }
        '?' => {
            state.current_token = Some(Token {
                kind: TokenKind::XmlDeclaration,
                attributes: HashMap::new(),
                data: ch.to_string(),
            });
            state.state = TokenState::XmlDeclaration;
        }
        '/' => {
            state.current_token = Some(Token {
                kind: TokenKind::EndTag,
                attributes: HashMap::new(),
                data: String::new(),
            });
            state.state = TokenState::EndTagOpen;
        }
        ch if ch.is_alphabetic() => {
            state.current_token = Some(Token {
                kind: TokenKind::StartTag,
                attributes: HashMap::new(),
                data: ch.to_string(),
            });
            state.state = TokenState::TagName;
        }
        _ => {
            state.state = TokenState::Data;
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

            state.state = TokenState::Data;
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
            state.state = TokenState::TagName;
        }
        _ => {
            state.state = TokenState::Data;
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
            state.state = TokenState::Data;
        }
        ch if ch.is_whitespace() => {}
        _ => {
            state.state = TokenState::BeforeAttributeName;
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
        '>' => handle_closing_tag(state, tokens),
        '/' => {
            state.state = TokenState::SelfClosingTagStart;
        }
        ch if ch.is_whitespace() => {
            state.state = TokenState::BeforeAttributeName;
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

/// Finalizes the current tag token and emits it.
///
/// # Arguments
/// * `state` - A mutable reference to the tokenizer state.
/// * `tokens` - A mutable reference to the vector of tokens to which new tokens will be emitted.
///
/// # Behavior
/// - Inserts the current attribute name and value into the current token's attributes.
/// - Clears the current attribute name and value.
/// - Transitions to the appropriate parser data state (ScriptData, StyleData or Data) based on the token's data.
/// - Emits the current token.
pub fn handle_closing_tag(state: &mut TokenizerState, tokens: &mut Vec<Token>) {
    if let Some(mut token) = state.current_token.take() {
        token.attributes.insert(
            state.current_attribute_name.clone(),
            state.current_attribute_value.clone(),
        );

        state.current_attribute_name.clear();
        state.current_attribute_value.clear();

        if token.data == "script" && token.kind != TokenKind::EndTag {
            state.state = TokenState::ScriptData;
        } else if token.data == "style" && token.kind != TokenKind::EndTag {
            state.state = TokenState::StyleData;
        } else {
            if token.data == "pre" {
                if token.kind == TokenKind::StartTag {
                    state.context.inside_preformatted = true;
                } else if token.kind == TokenKind::EndTag {
                    state.context.inside_preformatted = false;
                }
            }
            state.state = TokenState::Data;
        }

        HtmlTokenizer::emit_token(tokens, token);
    }
}
