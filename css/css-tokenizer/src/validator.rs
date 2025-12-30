use crate::{
    char::{is_digit, is_ident_start_code_point},
    tokenizer::CssTokenizer,
};

/// Check if two code points are a valid escape (§4.3.8)
///
/// # Arguments
/// * `first` - The first code point
/// * `second` - The second code point
pub fn two_code_points_are_valid_escape(first: Option<char>, second: Option<char>) -> bool {
    match (first, second) {
        (Some('\\'), Some('\n')) => false,
        (Some('\\'), _) => true,
        _ => false,
    }
}

/// Check if current position starts with a valid escape
///
/// # Arguments
/// * `tokenizer` - The CSS tokenizer
pub fn starts_with_valid_escape(tokenizer: &mut CssTokenizer) -> bool {
    two_code_points_are_valid_escape(tokenizer.stream.current, tokenizer.stream.peek())
}

/// Check if three code points would start an ident sequence (§4.3.9)
///
/// # Arguments
/// * `first` - The first code point
/// * `second` - The second code point
/// * `third` - The third code point
pub fn three_code_points_would_start_ident(
    first: Option<char>,
    second: Option<char>,
    third: Option<char>,
) -> bool {
    match first {
        Some('-') => {
            matches!(second, Some(c) if is_ident_start_code_point(c) || c == '-')
                || two_code_points_are_valid_escape(second, third)
        }
        Some(c) if is_ident_start_code_point(c) => true,
        Some('\\') => two_code_points_are_valid_escape(first, second),
        _ => false,
    }
}

/// Check if input stream starts with an ident sequence
///
/// # Arguments
/// * `tokenizer` - The CSS tokenizer
pub fn input_starts_with_ident_sequence(tokenizer: &mut CssTokenizer) -> bool {
    three_code_points_would_start_ident(
        tokenizer.stream.peek(),
        tokenizer.stream.peek_at(1),
        tokenizer.stream.peek_at(2),
    )
}

/// Check if three code points would start a number (§4.3.10)
///
/// # Arguments
/// * `first` - The first code point
/// * `second` - The second code point
/// * `third` - The third code point
fn three_code_points_would_start_number(
    first: Option<char>,
    second: Option<char>,
    third: Option<char>,
) -> bool {
    match first {
        Some('+') | Some('-') => {
            second.is_some_and(is_digit) || (second == Some('.') && third.is_some_and(is_digit))
        }
        Some('.') => second.is_some_and(is_digit),
        Some(c) if is_digit(c) => true,
        _ => false,
    }
}

/// Check if input stream starts with a number
///
/// # Arguments
/// * `tokenizer` - The CSS tokenizer
pub fn input_starts_with_number(tokenizer: &mut CssTokenizer) -> bool {
    three_code_points_would_start_number(
        tokenizer.stream.current,
        tokenizer.stream.peek(),
        tokenizer.stream.peek_at(1),
    )
}
