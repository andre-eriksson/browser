use crate::{
    char::{is_ident_code_point, is_whitespace},
    consumers::{string::consume_escaped_code_point, url::consume_url_token},
    tokenizer::CssTokenizer,
    tokens::CssToken,
    validator::two_code_points_are_valid_escape,
};

/// Consume an ident-like token (§4.3.4)
pub fn consume_ident_like_token(tokenizer: &mut CssTokenizer) -> CssToken {
    let string = consume_ident_sequence(tokenizer);

    // Check for url( special case
    if string.eq_ignore_ascii_case("url") && tokenizer.stream.peek() == Some('(') {
        tokenizer.stream.consume(); // consume (

        // Consume whitespace
        while tokenizer.stream.peek().is_some_and(is_whitespace)
            && tokenizer.stream.peek_at(1).is_some_and(is_whitespace)
        {
            tokenizer.stream.consume();
        }

        // Check for quoted URL (which becomes a function token)
        let next = tokenizer.stream.peek();
        let next2 = tokenizer.stream.peek_at(1);

        if next == Some('"')
            || next == Some('\'')
            || (next.is_some_and(is_whitespace) && (next2 == Some('"') || next2 == Some('\'')))
        {
            CssToken::Function(string)
        } else {
            consume_url_token(tokenizer)
        }
    } else if tokenizer.stream.peek() == Some('(') {
        tokenizer.stream.consume();
        CssToken::Function(string)
    } else {
        CssToken::Ident(string)
    }
}

/// Consume an ident sequence (§4.3.11)
pub fn consume_ident_sequence(tokenizer: &mut CssTokenizer) -> String {
    let mut result = String::new();

    loop {
        let c = match tokenizer.stream.peek() {
            Some(c) => c,
            None => return result,
        };

        match c {
            c if is_ident_code_point(c) => {
                tokenizer.stream.consume();
                result.push(c);
            }
            '\\' if two_code_points_are_valid_escape(Some('\\'), tokenizer.stream.peek_at(1)) => {
                tokenizer.stream.consume();
                result.push(consume_escaped_code_point(tokenizer));
            }
            _ => {
                return result;
            }
        }
    }
}
