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
