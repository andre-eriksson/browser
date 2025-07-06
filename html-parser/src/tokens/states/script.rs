use std::collections::HashMap;

use crate::tokens::{
    state::{ParserState, Token, TokenKind},
    tokenizer::HtmlTokenizer,
};

pub fn handle_script_data_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '<' => {
            // Handle the start of a script end tag
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
            tokenizer.temporary_buffer.push('>'); // Complete the end tag
            if tokenizer.temporary_buffer == expected {
                if let Some(token) = tokenizer.current_token.take() {
                    // Emit the script start tag token
                    tokenizer.emit_token(token);
                }

                // Emit a script end tag token
                tokenizer.emit_token(Token {
                    kind: TokenKind::EndTag,
                    attributes: HashMap::new(),
                    data: "script".to_string(),
                });

                tokenizer.temporary_buffer.clear();
                tokenizer.state = ParserState::Data; // Return to Data state
            } else {
                if let Some(token) = tokenizer.current_token.as_mut() {
                    tokenizer.temporary_buffer.push(ch);
                    token.data.push_str(&tokenizer.temporary_buffer);
                }
                tokenizer.temporary_buffer.clear();
                tokenizer.state = ParserState::ScriptData; // Return to ScriptData state
            }
        }
        ch if ch.is_whitespace() => {
            // Ignore whitespace in script end tag open state
        }
        _ => {
            if tokenizer.temporary_buffer.len() == expected.len() {
                if tokenizer.temporary_buffer != expected {
                    // If we have accumulated enough characters, emit the current token
                    if let Some(token) = tokenizer.current_token.as_mut() {
                        token.data.push_str(&tokenizer.temporary_buffer);
                        tokenizer.state = ParserState::ScriptData; // Return to ScriptData state
                        tokenizer.temporary_buffer.clear();
                    }
                }
            } else {
                // Continue accumulating characters for the script end tag
                tokenizer.temporary_buffer.push(ch);
            }
        }
    }
}
