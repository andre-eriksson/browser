use crate::{
    char::{is_digit, is_ident_code_point, is_ident_start_code_point, is_whitespace},
    consumers::{
        ident::{consume_ident_like_token, consume_ident_sequence},
        numeric::consume_numeric_token,
        string::consume_string_token,
    },
    tokenizer::CssTokenizer,
    tokens::{CssToken, CssTokenKind, HashType},
    validator::{
        input_starts_with_ident_sequence, input_starts_with_number, starts_with_valid_escape,
        three_code_points_would_start_ident, two_code_points_are_valid_escape,
    },
};
use errors::tokenization::CssTokenizationError;

/// Consume a token (§4.3.1)
pub(crate) fn consume_token(tokenizer: &mut CssTokenizer) -> CssToken {
    consume_comments(tokenizer);

    let c = match tokenizer.stream.consume() {
        Some(c) => c,
        None => {
            return CssToken {
                kind: CssTokenKind::Eof,
                position: Some(tokenizer.stream.position()),
            };
        }
    };

    match c {
        c if is_whitespace(c) => {
            consume_whitespace(tokenizer);
            CssToken {
                kind: CssTokenKind::Whitespace,
                position: Some(tokenizer.stream.position()),
            }
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
                CssToken {
                    kind: CssTokenKind::Hash { value, type_flag },
                    position: Some(tokenizer.stream.position()),
                }
            } else {
                CssToken {
                    kind: CssTokenKind::Delim('#'),
                    position: Some(tokenizer.stream.position()),
                }
            }
        }
        '\'' => consume_string_token(tokenizer, '\''),
        '(' => CssToken {
            kind: CssTokenKind::OpenParen,
            position: Some(tokenizer.stream.position()),
        },
        ')' => CssToken {
            kind: CssTokenKind::CloseParen,
            position: Some(tokenizer.stream.position()),
        },
        '+' => {
            if input_starts_with_number(tokenizer) {
                tokenizer.stream.reconsume();
                consume_numeric_token(tokenizer)
            } else {
                CssToken {
                    kind: CssTokenKind::Delim('+'),
                    position: Some(tokenizer.stream.position()),
                }
            }
        }
        ',' => CssToken {
            kind: CssTokenKind::Comma,
            position: Some(tokenizer.stream.position()),
        },
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
                CssToken {
                    kind: CssTokenKind::Cdc,
                    position: Some(tokenizer.stream.position()),
                }
            } else if input_starts_with_ident_sequence(tokenizer) {
                tokenizer.stream.reconsume();
                consume_ident_like_token(tokenizer)
            } else {
                CssToken {
                    kind: CssTokenKind::Delim('-'),
                    position: Some(tokenizer.stream.position()),
                }
            }
        }
        '.' => {
            if input_starts_with_number(tokenizer) {
                tokenizer.stream.reconsume();
                consume_numeric_token(tokenizer)
            } else {
                CssToken {
                    kind: CssTokenKind::Delim('.'),
                    position: Some(tokenizer.stream.position()),
                }
            }
        }
        ':' => CssToken {
            kind: CssTokenKind::Colon,
            position: Some(tokenizer.stream.position()),
        },
        ';' => CssToken {
            kind: CssTokenKind::Semicolon,
            position: Some(tokenizer.stream.position()),
        },
        '<' => {
            if tokenizer.stream.peek() == Some('!')
                && tokenizer.stream.peek_at(1) == Some('-')
                && tokenizer.stream.peek_at(2) == Some('-')
            {
                // CDO token <!--
                tokenizer.stream.consume(); // !
                tokenizer.stream.consume(); // -
                tokenizer.stream.consume(); // -
                CssToken {
                    kind: CssTokenKind::Cdo,
                    position: Some(tokenizer.stream.position()),
                }
            } else {
                CssToken {
                    kind: CssTokenKind::Delim('<'),
                    position: Some(tokenizer.stream.position()),
                }
            }
        }
        '@' => {
            let next = tokenizer.stream.peek();
            let next2 = tokenizer.stream.peek_at(1);
            let next3 = tokenizer.stream.peek_at(2);

            if three_code_points_would_start_ident(next, next2, next3) {
                let value = consume_ident_sequence(tokenizer);
                CssToken {
                    kind: CssTokenKind::AtKeyword(value),
                    position: Some(tokenizer.stream.position()),
                }
            } else {
                CssToken {
                    kind: CssTokenKind::Delim('@'),
                    position: Some(tokenizer.stream.position()),
                }
            }
        }
        '[' => CssToken {
            kind: CssTokenKind::OpenSquare,
            position: Some(tokenizer.stream.position()),
        },
        '\\' => {
            if starts_with_valid_escape(tokenizer) {
                tokenizer.stream.reconsume();
                consume_ident_like_token(tokenizer)
            } else {
                tokenizer.record_error(CssTokenizationError::InvalidEscape);
                CssToken {
                    kind: CssTokenKind::Delim('\\'),
                    position: Some(tokenizer.stream.position()),
                }
            }
        }
        ']' => CssToken {
            kind: CssTokenKind::CloseSquare,
            position: Some(tokenizer.stream.position()),
        },
        '{' => CssToken {
            kind: CssTokenKind::OpenCurly,
            position: Some(tokenizer.stream.position()),
        },
        '}' => CssToken {
            kind: CssTokenKind::CloseCurly,
            position: Some(tokenizer.stream.position()),
        },
        c if is_digit(c) => {
            tokenizer.stream.reconsume();
            consume_numeric_token(tokenizer)
        }
        c if is_ident_start_code_point(c) => {
            tokenizer.stream.reconsume();
            consume_ident_like_token(tokenizer)
        }
        _ => CssToken {
            kind: CssTokenKind::Delim(c),
            position: Some(tokenizer.stream.position()),
        },
    }
}

/// Consume comments (§4.3.2)
fn consume_comments(tokenizer: &mut CssTokenizer) {
    loop {
        if tokenizer.stream.peek() == Some('/') && tokenizer.stream.peek_at(1) == Some('*') {
            tokenizer.stream.consume();
            tokenizer.stream.consume();

            let mut found_end = false;
            while let Some(c) = tokenizer.stream.consume() {
                match c {
                    '*' if tokenizer.stream.peek() == Some('/') => {
                        tokenizer.stream.consume();
                        found_end = true;
                        break;
                    }
                    _ => {}
                }
            }

            if !found_end {
                tokenizer.record_error(CssTokenizationError::EofInComment);
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
