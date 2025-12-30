/// Check if a character is a digit (§4.2)
pub fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

/// Check if a character is a hex digit (§4.2)
pub fn is_hex_digit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

/// Check if a character is an ident-start code point (§4.2)
pub fn is_ident_start_code_point(c: char) -> bool {
    c.is_ascii_alphabetic() || c as u32 >= 0x80 || c == '_'
}

/// Check if a character is an ident code point (§4.2)
pub fn is_ident_code_point(c: char) -> bool {
    is_ident_start_code_point(c) || is_digit(c) || c == '-'
}

/// Check if a character is a non-printable code point (§4.2)
pub fn is_non_printable(c: char) -> bool {
    let code = c as u32;
    (0x00..=0x08).contains(&code) || code == 0x0B || (0x0E..=0x1F).contains(&code) || code == 0x7F
}

/// Check if a character is whitespace (§4.2)
pub fn is_whitespace(c: char) -> bool {
    c == '\n' || c == '\t' || c == ' '
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_digit() {
        assert!(is_digit('0'));
        assert!(is_digit('5'));
        assert!(is_digit('9'));
        assert!(!is_digit('a'));
    }

    #[test]
    fn test_is_hex_digit() {
        assert!(is_hex_digit('0'));
        assert!(is_hex_digit('9'));
        assert!(is_hex_digit('a'));
        assert!(is_hex_digit('F'));
        assert!(!is_hex_digit('g'));
    }

    #[test]
    fn test_is_ident_start_code_point() {
        assert!(is_ident_start_code_point('a'));
        assert!(is_ident_start_code_point('Z'));
        assert!(is_ident_start_code_point('_'));
        assert!(is_ident_start_code_point('é')); // non-ASCII
        assert!(!is_ident_start_code_point('1'));
    }

    #[test]
    fn test_is_ident_code_point() {
        assert!(is_ident_code_point('a'));
        assert!(is_ident_code_point('Z'));
        assert!(is_ident_code_point('_'));
        assert!(is_ident_code_point('é')); // non-ASCII
        assert!(is_ident_code_point('1'));
        assert!(is_ident_code_point('-'));
        assert!(!is_ident_code_point('@'));
    }

    #[test]
    fn test_is_non_printable() {
        assert!(is_non_printable('\x00'));
        assert!(is_non_printable('\x07'));
        assert!(is_non_printable('\x0B'));
        assert!(is_non_printable('\x1F'));
        assert!(is_non_printable('\x7F'));
        assert!(!is_non_printable('a'));
    }

    #[test]
    fn test_is_whitespace() {
        assert!(is_whitespace('\n'));
        assert!(is_whitespace('\t'));
        assert!(is_whitespace(' '));
        assert!(!is_whitespace('a'));
    }
}
