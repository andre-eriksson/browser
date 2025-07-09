use std::collections::HashMap;

use crate::tokens::{
    state::{ParserState, Token, TokenKind},
    tokenizer::HtmlTokenizer,
};

pub fn handle_script_data_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '<' => {
            tokenizer.temporary_buffer.clear();
            tokenizer.temporary_buffer.push(ch);
            tokenizer.state = ParserState::ScriptDataEndTagOpen;
        }
        _ => {
            if let Some(token) = tokenizer.current_token.as_mut() {
                token.data.push(ch);
            } else {
                tokenizer.current_token = Some(Token {
                    kind: TokenKind::Text,
                    attributes: HashMap::new(),
                    data: ch.to_string(),
                });
            }
        }
    }
}

pub fn handle_script_data_end_tag_open_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    let expected = "</script>";
    match ch {
        '>' => {
            tokenizer.temporary_buffer.push('>');
            if tokenizer.temporary_buffer == expected {
                if let Some(token) = tokenizer.current_token.take() {
                    tokenizer.emit_token(token);
                }

                tokenizer.emit_token(Token {
                    kind: TokenKind::EndTag,
                    attributes: HashMap::new(),
                    data: "script".to_string(),
                });

                tokenizer.temporary_buffer.clear();
                tokenizer.state = ParserState::Data;
            } else {
                if let Some(token) = tokenizer.current_token.as_mut() {
                    tokenizer.temporary_buffer.push(ch);
                    token.data.push_str(&tokenizer.temporary_buffer);
                }
                tokenizer.temporary_buffer.clear();
                tokenizer.state = ParserState::ScriptData;
            }
        }
        ch if ch.is_whitespace() => {}
        _ => {
            if tokenizer.temporary_buffer.len() == expected.len() {
                if tokenizer.temporary_buffer != expected {
                    if let Some(token) = tokenizer.current_token.as_mut() {
                        token.data.push_str(&tokenizer.temporary_buffer);
                        tokenizer.state = ParserState::ScriptData;
                        tokenizer.temporary_buffer.clear();
                    }
                }
            } else {
                tokenizer.temporary_buffer.push(ch);
            }
        }
    }
}
