use crate::errors::CssTokenizationError;
use crate::{
    char::{is_non_printable, is_whitespace},
    consumers::{string::consume_escaped_code_point, token::consume_whitespace},
    tokenizer::CssTokenizer,
    tokens::{CssToken, CssTokenKind},
    validator::starts_with_valid_escape,
};

/// Consume a URL token (§4.3.6)
pub(crate) fn consume_url_token(tokenizer: &mut CssTokenizer) -> CssToken {
    consume_whitespace(tokenizer);
    let mut value = String::new();

    loop {
        let c = match tokenizer.stream.consume() {
            Some(c) => c,
            None => {
                tokenizer.record_error(CssTokenizationError::EofInUrl);
                return CssToken {
                    kind: CssTokenKind::Url(value),
                    position: CssTokenizer::collect_positions(tokenizer),
                };
            }
        };

        match c {
            ')' => {
                return CssToken {
                    kind: CssTokenKind::Url(value),
                    position: CssTokenizer::collect_positions(tokenizer),
                };
            }
            c if is_whitespace(c) => {
                consume_whitespace(tokenizer);

                let ch = match tokenizer.stream.peek() {
                    Some(c) => c,
                    None => {
                        tokenizer.record_error(CssTokenizationError::EofInUrl);
                        return CssToken {
                            kind: CssTokenKind::Url(value),
                            position: CssTokenizer::collect_positions(tokenizer),
                        };
                    }
                };

                match ch {
                    ')' => {
                        tokenizer.stream.consume();
                        return CssToken {
                            kind: CssTokenKind::Url(value),
                            position: CssTokenizer::collect_positions(tokenizer),
                        };
                    }
                    _ => {
                        consume_bad_url_remnants(tokenizer);
                        return CssToken {
                            kind: CssTokenKind::BadUrl,
                            position: CssTokenizer::collect_positions(tokenizer),
                        };
                    }
                }
            }
            '"' | '\'' | '(' => {
                tokenizer.record_error_at_current_char(CssTokenizationError::InvalidCharacterInUrl);
                consume_bad_url_remnants(tokenizer);
                return CssToken {
                    kind: CssTokenKind::BadUrl,
                    position: CssTokenizer::collect_positions(tokenizer),
                };
            }
            c if is_non_printable(c) => {
                tokenizer.record_error_at_current_char(CssTokenizationError::InvalidCharacterInUrl);
                consume_bad_url_remnants(tokenizer);
                return CssToken {
                    kind: CssTokenKind::BadUrl,
                    position: CssTokenizer::collect_positions(tokenizer),
                };
            }
            '\\' => {
                if starts_with_valid_escape(tokenizer) {
                    value.push(consume_escaped_code_point(tokenizer));
                } else {
                    tokenizer.record_error(CssTokenizationError::InvalidEscapeInUrl);
                    consume_bad_url_remnants(tokenizer);
                    return CssToken {
                        kind: CssTokenKind::BadUrl,
                        position: CssTokenizer::collect_positions(tokenizer),
                    };
                }
            }
            c => value.push(c),
        }
    }
}

/// Consume the remnants of a bad URL (§4.3.14)
fn consume_bad_url_remnants(tokenizer: &mut CssTokenizer) {
    loop {
        let c = match tokenizer.stream.consume() {
            Some(c) => c,
            None => return,
        };

        match c {
            ')' => return,
            '\\' if tokenizer.stream.peek().is_some() => {
                consume_escaped_code_point(tokenizer);
            }
            _ => {}
        }
    }
}
