use std::collections::HashMap;

use html_syntax::token::{Token, TokenKind};

use crate::tokens::{state::ParserState, tokenizer::HtmlTokenizer};

/// Handles the script data state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is '<', it prepares to check for an end tag and transitions to the `ParserState::ScriptDataEndTagOpen` state.
/// - For any other character, it appends the character to the current script data token's data.
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

/// Handles the script data end tag open state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is '>', it checks if the temporary buffer matches the expected end tag "</script>".
///   - If it matches, it emits the current token and an end tag token for "script", then transitions to the `ParserState::Data` state.
///   - If it doesn't match, it appends the temporary buffer to the current token's data and transitions back to the `ParserState::ScriptData` state.
/// - If the character is whitespace, it ignores it.
/// - For any other character, it appends it to the temporary buffer until it reaches the length of the expected end tag.
///   - If the temporary buffer does not match the expected end tag, it appends it to the current token's data and transitions back to the `ParserState::ScriptData` state.
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
                if tokenizer.temporary_buffer != expected
                    && let Some(token) = tokenizer.current_token.as_mut()
                {
                    token.data.push_str(&tokenizer.temporary_buffer);
                    tokenizer.state = ParserState::ScriptData;
                    tokenizer.temporary_buffer.clear();
                }
            } else {
                tokenizer.temporary_buffer.push(ch);
            }
        }
    }
}
