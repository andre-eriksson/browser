use std::collections::HashMap;

use crate::tokens::{
    state::{ParserState, Token, TokenKind},
    tokenizer::HtmlTokenizer,
};

/// Handles the start of a tag, after a `<`
pub fn handle_tag_open_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '!' => {
            tokenizer.state = ParserState::StartDeclaration;
        }
        '?' => {
            tokenizer.current_token = Some(Token {
                kind: TokenKind::XmlDeclaration,
                attributes: HashMap::new(),
                data: ch.to_string(),
            });
            tokenizer.state = ParserState::XmlDeclaration;
        }
        '/' => {
            tokenizer.current_token = Some(Token {
                kind: TokenKind::EndTag,
                attributes: HashMap::new(),
                data: String::new(),
            });
            tokenizer.state = ParserState::EndTagOpen;
        }
        ch if ch.is_alphabetic() => {
            tokenizer.current_token = Some(Token {
                kind: TokenKind::StartTag,
                attributes: HashMap::new(),
                data: ch.to_string(),
            });
            tokenizer.state = ParserState::TagName;
        }
        _ => {
            tokenizer.state = ParserState::Data;
        }
    }
}

/// Handles the start of an end tag, after a `</`
pub fn handle_end_tag_open_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '>' => {
            if let Some(token) = tokenizer.current_token.take() {
                tokenizer.emit_token(token);
            }

            tokenizer.state = ParserState::Data;
        }
        ch if ch.is_alphabetic() => {
            if let Some(token) = tokenizer.current_token.as_mut() {
                token.data.push(ch);
            } else {
                tokenizer.current_token = Some(Token {
                    kind: TokenKind::EndTag,
                    attributes: HashMap::new(),
                    data: ch.to_string(),
                });
            }
            tokenizer.state = ParserState::TagName;
        }
        _ => {
            tokenizer.state = ParserState::Data;
        }
    }
}

/// Handles the start of a self-closing tag, after a `<test /`
pub fn handle_self_closing_tag_start_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '>' => {
            if let Some(mut token) = tokenizer.current_token.take() {
                if !tokenizer.current_attribute_name.is_empty() {
                    token.attributes.insert(
                        tokenizer.current_attribute_name.clone(),
                        tokenizer.current_attribute_value.clone(),
                    );

                    tokenizer.current_attribute_name.clear();
                    tokenizer.current_attribute_value.clear();
                }

                tokenizer.emit_token(token);
            }
            tokenizer.state = ParserState::Data;
        }
        ch if ch.is_whitespace() => {}
        _ => {
            tokenizer.state = ParserState::BeforeAttributeName;
        }
    }
}

/// Handles the tag name state, after the `<`
pub fn handle_tag_name_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '>' => {
            if let Some(token) = tokenizer.current_token.take() {
                if token.data == "script" {
                    tokenizer.state = ParserState::ScriptData;
                } else {
                    if token.data == "pre" {
                        if token.kind == TokenKind::StartTag {
                            tokenizer.context.inside_preformatted = true;
                        } else if token.kind == TokenKind::EndTag {
                            tokenizer.context.inside_preformatted = false;
                        }
                    }

                    tokenizer.state = ParserState::Data;
                }

                tokenizer.emit_token(token);
            }
        }
        '/' => {
            tokenizer.state = ParserState::SelfClosingTagStart;
        }
        ch if ch.is_whitespace() => {
            tokenizer.state = ParserState::BeforeAttributeName;
        }
        _ => {
            if let Some(token) = tokenizer.current_token.as_mut() {
                token.data.push(ch);
            } else {
                tokenizer.current_token = Some(Token {
                    kind: TokenKind::StartTag,
                    attributes: HashMap::new(),
                    data: ch.to_string(),
                });
            }
        }
    }
}
