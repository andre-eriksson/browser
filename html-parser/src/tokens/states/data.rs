use std::collections::HashMap;

use crate::tokens::{
    state::{ParserState, Token, TokenKind},
    tokenizer::HtmlTokenizer,
};

/// Checks if the current context is within a preformatted tag, like <pre> or <textarea>.
/// This is used to determine if whitespace should be preserved exactly.
pub fn is_in_preformatted_context(tokenizer: &HtmlTokenizer) -> bool {
    // Track open tags to determine if we're inside a preformatted context
    let mut tag_stack = Vec::new();

    for token in &tokenizer.tokens {
        match &token.kind {
            TokenKind::StartTag => {
                if token.data == "pre" || token.data == "textarea" {
                    tag_stack.push(token.data.clone());
                }
            }
            TokenKind::EndTag => {
                if token.data == "pre" || token.data == "textarea" {
                    if let Some(last_tag) = tag_stack.last() {
                        if last_tag == &token.data {
                            tag_stack.pop();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    !tag_stack.is_empty()
}

/// Preserves significant whitespace in the text.
pub fn preserve_significant_whitespace(tokenizer: &HtmlTokenizer) -> String {
    let text = tokenizer.temporary_buffer.clone();
    // Check if we're inside a <pre> or <code> tag by looking at current token context
    let preserve_exact = is_in_preformatted_context(tokenizer);

    if preserve_exact {
        // For preformatted content, preserve all whitespace exactly including \r\n
        return text.trim_start().trim_end().to_string();
    }

    // If the text is only whitespace, preserve it as a single space
    // This is important for whitespace between tags like "</span> world"
    if text.trim().is_empty() && !text.is_empty() {
        return " ".to_string();
    }

    // For text with actual content, do minimal normalization
    // Replace sequences of whitespace within the text with single spaces
    // but preserve leading/trailing whitespace as single spaces if present
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
                if !processed_text.is_empty() {
                    // Emit the data token if there's accumulated data
                    tokenizer.emit_token(Token {
                        kind: TokenKind::Text,
                        attributes: HashMap::new(),
                        data: processed_text,
                    });
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
