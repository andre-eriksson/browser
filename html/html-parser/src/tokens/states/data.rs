use std::collections::HashMap;

use html_syntax::token::{Token, TokenKind};

use crate::tokens::{
    state::ParserState,
    tokenizer::{HtmlTokenizer, TokenizerState},
};

/// Preserves significant whitespace in text nodes according to HTML parsing rules.
///
/// # Arguments
/// * `tokenizer` - A reference to the HTML tokenizer.
///
/// # Returns
/// * `String` - The processed text with significant whitespace preserved.
pub fn preserve_significant_whitespace(state: &TokenizerState) -> String {
    let text = state.temporary_buffer.clone();

    if state.context.inside_preformatted {
        return text;
    }

    if state.current_token.is_some() && text.trim().is_empty() && !text.is_empty() {
        return " ".to_string();
    }

    let has_leading_ws = text.starts_with(char::is_whitespace);
    let has_trailing_ws = text.ends_with(char::is_whitespace);
    let has_leading_newline = text.starts_with('\n') || text.starts_with('\r');
    let has_trailing_newline = text.ends_with('\n') || text.ends_with('\r');

    let normalized_middle = text.split_whitespace().collect::<Vec<_>>().join(" ");

    let mut result = String::new();
    if has_leading_ws && !normalized_middle.is_empty() && !has_leading_newline {
        result.push(' ');
    }
    result.push_str(&normalized_middle);
    if has_trailing_ws && !normalized_middle.is_empty() && !has_trailing_newline {
        result.push(' ');
    }

    result
}

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
                let processed_text = preserve_significant_whitespace(state);

                if state.context.inside_preformatted {
                    let mut current_pos = 0;
                    let chars: Vec<char> = processed_text.chars().collect();

                    while current_pos < chars.len() {
                        let mut line = String::new();

                        while current_pos < chars.len() {
                            let ch = chars[current_pos];
                            current_pos += 1;

                            if ch == '\n' {
                                line.push(ch);
                                break;
                            } else if ch == '\r' {
                                line.push(ch);
                                if current_pos < chars.len() && chars[current_pos] == '\n' {
                                    line.push(chars[current_pos]);
                                    current_pos += 1;
                                }
                                break;
                            } else {
                                line.push(ch);
                            }
                        }

                        if !line.is_empty() {
                            HtmlTokenizer::emit_token(
                                tokens,
                                Token {
                                    kind: TokenKind::Text,
                                    attributes: HashMap::new(),
                                    data: line,
                                },
                            );
                        }
                    }
                } else if !processed_text.is_empty() {
                    HtmlTokenizer::emit_token(
                        tokens,
                        Token {
                            kind: TokenKind::Text,
                            attributes: HashMap::new(),
                            data: processed_text,
                        },
                    );
                }

                state.temporary_buffer.clear();
            }
            state.state = ParserState::TagOpen;
        }
        _ => {
            state.temporary_buffer.push(ch);
        }
    }
}
