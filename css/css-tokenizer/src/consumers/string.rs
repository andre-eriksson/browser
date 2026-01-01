use crate::{
    char::is_hex_digit, consumers::token::consume_whitespace, tokenizer::CssTokenizer,
    tokens::CssToken,
};
use errors::tokenization::CssTokenizationError;

/// Consume a string token (§4.3.5)
pub(crate) fn consume_string_token(tokenizer: &mut CssTokenizer, ending: char) -> CssToken {
    let mut value = String::new();

    loop {
        let c = match tokenizer.stream.consume() {
            Some(c) => c,
            None => {
                tokenizer.record_error(CssTokenizationError::EofInString);
                return CssToken::String(value);
            }
        };

        match c {
            c if c == ending => return CssToken::String(value),
            '\n' => {
                tokenizer.record_error_at_current_char(CssTokenizationError::NewlineInString);
                tokenizer.stream.reconsume();
                return CssToken::BadString;
            }
            '\\' => match tokenizer.stream.peek() {
                None => {}
                Some('\n') => {
                    tokenizer.stream.consume();
                }
                _ => {
                    value.push(consume_escaped_code_point(tokenizer));
                }
            },
            c => {
                value.push(c);
            }
        }
    }
}

/// Consume an escaped code point (§4.3.7)
pub(crate) fn consume_escaped_code_point(tokenizer: &mut CssTokenizer) -> char {
    let c = match tokenizer.stream.consume() {
        Some(c) => c,
        None => return '\u{FFFD}',
    };

    match c {
        c if is_hex_digit(c) => {
            let mut hex_value = c.to_digit(16).unwrap();

            for _ in 0..5 {
                if let Some(next) = tokenizer.stream.peek() {
                    if is_hex_digit(next) {
                        tokenizer.stream.consume();
                        hex_value = hex_value * 16 + next.to_digit(16).unwrap();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            consume_whitespace(tokenizer);

            if hex_value == 0 || (0xD800..=0xDFFF).contains(&hex_value) || hex_value > 0x10FFFF {
                '\u{FFFD}'
            } else {
                char::from_u32(hex_value).unwrap_or('\u{FFFD}')
            }
        }
        c => c,
    }
}
