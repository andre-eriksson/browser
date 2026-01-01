use crate::{
    char::{is_digit, is_ident_code_point, is_ident_start_code_point, is_whitespace},
    consumers::{
        ident::{consume_ident_like_token, consume_ident_sequence},
        numeric::consume_numeric_token,
        string::consume_string_token,
    },
    tokenizer::CssTokenizer,
    tokens::{CssToken, HashType},
    validator::{
        input_starts_with_ident_sequence, input_starts_with_number, starts_with_valid_escape,
        three_code_points_would_start_ident, two_code_points_are_valid_escape,
    },
};

/// Consume a token (§4.3.1)
pub(crate) fn consume_token(tokenizer: &mut CssTokenizer) -> CssToken {
    consume_comments(tokenizer);

    let c = match tokenizer.stream.consume() {
        Some(c) => c,
        None => return CssToken::Eof,
    };

    match c {
        c if is_whitespace(c) => {
            consume_whitespace(tokenizer);
            CssToken::Whitespace
        }
        '"' => consume_string_token(tokenizer, '"'),
        '#' => {
            let next = tokenizer.stream.peek();
            let next2 = tokenizer.stream.peek_at(1);

            if next.is_some_and(is_ident_code_point)
                || two_code_points_are_valid_escape(next, next2)
            {
                let type_flag = if three_code_points_would_start_ident(
                    next,
                    next2,
                    tokenizer.stream.peek_at(2),
                ) {
                    HashType::Id
                } else {
                    HashType::Unrestricted
                };

                let value = consume_ident_sequence(tokenizer);
                CssToken::Hash { value, type_flag }
            } else {
                CssToken::Delim('#')
            }
        }
        '\'' => consume_string_token(tokenizer, '\''),
        '(' => CssToken::OpenParen,
        ')' => CssToken::CloseParen,
        '+' => {
            if input_starts_with_number(tokenizer) {
                tokenizer.stream.reconsume();
                consume_numeric_token(tokenizer)
            } else {
                CssToken::Delim('+')
            }
        }
        ',' => CssToken::Comma,
        '-' => {
            if input_starts_with_number(tokenizer) {
                tokenizer.stream.reconsume();
                consume_numeric_token(tokenizer)
            } else if tokenizer.stream.peek() == Some('-')
                && tokenizer.stream.peek_at(1) == Some('>')
            {
                // CDC token -->
                tokenizer.stream.consume(); // -
                tokenizer.stream.consume(); // >
                CssToken::Cdc
            } else if input_starts_with_ident_sequence(tokenizer) {
                tokenizer.stream.reconsume();
                consume_ident_like_token(tokenizer)
            } else {
                CssToken::Delim('-')
            }
        }
        '.' => {
            if input_starts_with_number(tokenizer) {
                tokenizer.stream.reconsume();
                consume_numeric_token(tokenizer)
            } else {
                CssToken::Delim('.')
            }
        }
        ':' => CssToken::Colon,
        ';' => CssToken::Semicolon,
        '<' => {
            if tokenizer.stream.peek() == Some('!')
                && tokenizer.stream.peek_at(1) == Some('-')
                && tokenizer.stream.peek_at(2) == Some('-')
            {
                // CDO token <!--
                tokenizer.stream.consume(); // !
                tokenizer.stream.consume(); // -
                tokenizer.stream.consume(); // -
                CssToken::Cdo
            } else {
                CssToken::Delim('<')
            }
        }
        '@' => {
            let next = tokenizer.stream.peek();
            let next2 = tokenizer.stream.peek_at(1);
            let next3 = tokenizer.stream.peek_at(2);

            if three_code_points_would_start_ident(next, next2, next3) {
                let value = consume_ident_sequence(tokenizer);
                CssToken::AtKeyword(value)
            } else {
                CssToken::Delim('@')
            }
        }
        '[' => CssToken::OpenSquare,
        '\\' => {
            if starts_with_valid_escape(tokenizer) {
                tokenizer.stream.reconsume();
                consume_ident_like_token(tokenizer)
            } else {
                // Parse error
                CssToken::Delim('\\')
            }
        }
        ']' => CssToken::CloseSquare,
        '{' => CssToken::OpenCurly,
        '}' => CssToken::CloseCurly,
        c if is_digit(c) => {
            tokenizer.stream.reconsume();
            consume_numeric_token(tokenizer)
        }
        c if is_ident_start_code_point(c) => {
            tokenizer.stream.reconsume();
            consume_ident_like_token(tokenizer)
        }
        _ => CssToken::Delim(c),
    }
}

/// Consume comments (§4.3.2)
fn consume_comments(tokenizer: &mut CssTokenizer) {
    loop {
        if tokenizer.stream.peek() == Some('/') && tokenizer.stream.peek_at(1) == Some('*') {
            tokenizer.stream.consume();
            tokenizer.stream.consume();

            while let Some(c) = tokenizer.stream.consume() {
                match c {
                    '*' if tokenizer.stream.peek() == Some('/') => {
                        tokenizer.stream.consume();
                        break;
                    }
                    _ => {}
                }
            }
        } else {
            break;
        }
    }
}

/// Consume whitespace
pub(crate) fn consume_whitespace(tokenizer: &mut CssTokenizer) {
    while tokenizer.stream.peek().is_some_and(is_whitespace) {
        tokenizer.stream.consume();
    }
}
