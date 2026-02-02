use std::collections::HashMap;

use crate::{
    Token, TokenKind,
    state::TokenState,
    tokenizer::{HtmlTokenizer, TokenizerState},
};

/// Handles the data state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
/// * `tokens` - A mutable reference to the vector of tokens to which new tokens will be emitted.
///
/// # Behavior
/// - If the character is '<', it processes the temporary buffer and transitions to the `ParserState::TagOpen` state.
/// - For any other character, it appends the character to the temporary buffer.
pub fn handle_data_state(state: &mut TokenizerState, ch: char, tokens: &mut Vec<Token>) {
    match ch {
        '<' => {
            if !state.temporary_buffer.is_empty() {
                HtmlTokenizer::emit_token(
                    tokens,
                    Token {
                        kind: TokenKind::Text,
                        attributes: HashMap::new(),
                        data: state.temporary_buffer.clone(),
                    },
                );

                state.temporary_buffer.clear();
            }
            state.state = TokenState::TagOpen;
        }
        _ => {
            state.temporary_buffer.push(ch);
        }
    }
}
