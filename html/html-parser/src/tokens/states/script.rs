use std::collections::HashMap;

use html_syntax::token::{Token, TokenKind};

use crate::tokens::{
    state::ParserState,
    tokenizer::{HtmlTokenizer, TokenizerState},
};

/// Handles the script data state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is '<', it prepares to check for an end tag and transitions to the `ParserState::ScriptDataEndTagOpen` state.
/// - For any other character, it appends the character to the current script data token's data.
pub fn handle_script_data_state(state: &mut TokenizerState, ch: char) {
    match ch {
        '<' => {
            state.temporary_buffer.clear();
            state.temporary_buffer.push(ch);
            state.state = ParserState::ScriptDataEndTagOpen;
        }
        _ => {
            if let Some(token) = state.current_token.as_mut() {
                token.data.push(ch);
            } else {
                state.current_token = Some(Token {
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
/// * `tokens` - A mutable reference to the vector of tokens to which new tokens will be emitted.
///
/// # Behavior
/// - If the character is '>', it checks if the temporary buffer matches the expected end tag "</script>".
///   - If it matches, it emits the current token and an end tag token for "script", then transitions to the `ParserState::Data` state.
///   - If it doesn't match, it appends the temporary buffer to the current token's data and transitions back to the `ParserState::ScriptData` state.
/// - If the character is whitespace, it ignores it.
/// - For any other character, it appends it to the temporary buffer until it reaches the length of the expected end tag.
///   - If the temporary buffer does not match the expected end tag, it appends it to the current token's data and transitions back to the `ParserState::ScriptData` state.
pub fn handle_script_data_end_tag_open_state(
    state: &mut TokenizerState,
    ch: char,
    tokens: &mut Vec<Token>,
) {
    let expected = "</script>";
    match ch {
        '>' => {
            state.temporary_buffer.push('>');
            if state.temporary_buffer == expected {
                if let Some(token) = state.current_token.take() {
                    HtmlTokenizer::emit_token(tokens, token);
                }

                HtmlTokenizer::emit_token(
                    tokens,
                    Token {
                        kind: TokenKind::EndTag,
                        attributes: HashMap::new(),
                        data: "script".to_string(),
                    },
                );

                state.temporary_buffer.clear();
                state.state = ParserState::Data;
            } else {
                if let Some(token) = state.current_token.as_mut() {
                    state.temporary_buffer.push(ch);
                    token.data.push_str(&state.temporary_buffer);
                }
                state.temporary_buffer.clear();
                state.state = ParserState::ScriptData;
            }
        }
        ch if ch.is_whitespace() => {}
        _ => {
            if state.temporary_buffer.len() == expected.len() {
                if state.temporary_buffer != expected
                    && let Some(token) = state.current_token.as_mut()
                {
                    token.data.push_str(&state.temporary_buffer);
                    state.state = ParserState::ScriptData;
                    state.temporary_buffer.clear();
                }
            } else {
                state.temporary_buffer.push(ch);
            }
        }
    }
}
