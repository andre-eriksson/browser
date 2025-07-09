use std::collections::HashMap;

use crate::tokens::{
    state::{ParserState, Token, TokenKind},
    tokenizer::HtmlTokenizer,
};

/// Preserves significant whitespace in the text.
pub fn preserve_significant_whitespace(tokenizer: &HtmlTokenizer) -> String {
    let text = tokenizer.temporary_buffer.clone();

    if tokenizer.context.inside_preformatted {
        return text;
    }

    if tokenizer.current_token.is_some() && text.trim().is_empty() && !text.is_empty() {
        return " ".to_string();
    }

    let has_leading_ws = text.starts_with(char::is_whitespace);
    let has_trailing_ws = text.ends_with(char::is_whitespace);
    let has_leading_newline = text.starts_with('\n') || text.starts_with('\r');
    let has_trailing_newline = text.ends_with('\n') || text.ends_with('\r');

    let normalized_middle = text.trim().split_whitespace().collect::<Vec<_>>().join(" ");

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

pub fn handle_data_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '<' => {
            if !tokenizer.temporary_buffer.is_empty() {
                let processed_text = preserve_significant_whitespace(tokenizer);

                if tokenizer.context.inside_preformatted {
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
                            tokenizer.emit_token(Token {
                                kind: TokenKind::Text,
                                attributes: HashMap::new(),
                                data: line,
                            });
                        }
                    }
                } else {
                    if !processed_text.is_empty() {
                        tokenizer.emit_token(Token {
                            kind: TokenKind::Text,
                            attributes: HashMap::new(),
                            data: processed_text,
                        });
                    }
                }

                tokenizer.temporary_buffer.clear();
            }
            tokenizer.state = ParserState::TagOpen;
        }
        _ => {
            tokenizer.temporary_buffer.push(ch);
        }
    }
}
