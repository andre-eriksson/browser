use crate::{
    char::{is_non_printable, is_whitespace},
    consumers::{string::consume_escaped_code_point, token::consume_whitespace},
    tokenizer::CssTokenizer,
    tokens::CssToken,
    validator::starts_with_valid_escape,
};

/// Consume a URL token (§4.3.6)
pub fn consume_url_token(tokenizer: &mut CssTokenizer) -> CssToken {
    consume_whitespace(tokenizer);
    let mut value = String::new();

    loop {
        let c = match tokenizer.stream.consume() {
            Some(c) => c,
            None => return CssToken::Url(value),
        };

        match c {
            ')' => {
                return CssToken::Url(value);
            }
            c if is_whitespace(c) => {
                consume_whitespace(tokenizer);

                let ch = match tokenizer.stream.peek() {
                    Some(c) => c,
                    None => return CssToken::Url(value),
                };

                match ch {
                    ')' => {
                        tokenizer.stream.consume();
                        return CssToken::Url(value);
                    }
                    _ => {
                        consume_bad_url_remnants(tokenizer);
                        return CssToken::BadUrl;
                    }
                }
            }
            '"' | '\'' | '(' => {
                consume_bad_url_remnants(tokenizer);
                return CssToken::BadUrl;
            }
            c if is_non_printable(c) => {
                consume_bad_url_remnants(tokenizer);
                return CssToken::BadUrl;
            }
            '\\' => {
                if starts_with_valid_escape(tokenizer) {
                    value.push(consume_escaped_code_point(tokenizer));
                } else {
                    consume_bad_url_remnants(tokenizer);
                    return CssToken::BadUrl;
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
