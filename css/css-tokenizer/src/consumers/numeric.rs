use crate::{
    char::is_digit,
    consumers::ident::consume_ident_sequence,
    tokenizer::CssTokenizer,
    tokens::{CssToken, CssTokenKind, NumberType, NumericValue},
    validator::three_code_points_would_start_ident,
};

/// Consume a numeric token (§4.3.3)
pub(crate) fn consume_numeric_token(tokenizer: &mut CssTokenizer) -> CssToken {
    let number = consume_number(tokenizer);

    let next = tokenizer.stream.peek();
    let next2 = tokenizer.stream.peek_at(1);
    let next3 = tokenizer.stream.peek_at(2);

    if three_code_points_would_start_ident(next, next2, next3) {
        let unit = consume_ident_sequence(tokenizer);
        CssToken {
            kind: CssTokenKind::Dimension {
                value: number,
                unit,
            },
            position: Some(tokenizer.stream.position()),
        }
    } else if tokenizer.stream.peek() == Some('%') {
        tokenizer.stream.consume();
        CssToken {
            kind: CssTokenKind::Percentage(number),
            position: Some(tokenizer.stream.position()),
        }
    } else {
        CssToken {
            kind: CssTokenKind::Number(number),
            position: Some(tokenizer.stream.position()),
        }
    }
}

/// Consume a number (§4.3.12)
fn consume_number(tokenizer: &mut CssTokenizer) -> NumericValue {
    let mut repr = String::new();
    let mut type_flag = NumberType::Integer;

    // Optional sign
    if matches!(tokenizer.stream.peek(), Some('+') | Some('-')) {
        repr.push(tokenizer.stream.consume().unwrap());
    }

    // Integer part
    while tokenizer.stream.peek().is_some_and(is_digit) {
        repr.push(tokenizer.stream.consume().unwrap());
    }

    // Decimal part
    if tokenizer.stream.peek() == Some('.') && tokenizer.stream.peek_at(1).is_some_and(is_digit) {
        repr.push(tokenizer.stream.consume().unwrap()); // .
        type_flag = NumberType::Number;

        while tokenizer.stream.peek().is_some_and(is_digit) {
            repr.push(tokenizer.stream.consume().unwrap());
        }
    }

    // Exponent part
    if matches!(tokenizer.stream.peek(), Some('e') | Some('E')) {
        let next = tokenizer.stream.peek_at(1);
        let next2 = tokenizer.stream.peek_at(2);

        let has_exponent = if matches!(next, Some('+') | Some('-')) {
            next2.is_some_and(is_digit)
        } else {
            next.is_some_and(is_digit)
        };

        if has_exponent {
            repr.push(tokenizer.stream.consume().unwrap()); // e or E
            type_flag = NumberType::Number;

            if matches!(tokenizer.stream.peek(), Some('+') | Some('-')) {
                repr.push(tokenizer.stream.consume().unwrap());
            }

            while tokenizer.stream.peek().is_some_and(is_digit) {
                repr.push(tokenizer.stream.consume().unwrap());
            }
        }
    }

    let value = repr.parse::<f64>().unwrap_or(0.0);
    NumericValue::new(value, repr, type_flag)
}
