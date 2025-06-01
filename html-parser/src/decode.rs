use std::iter::Peekable;

pub struct Decoder<'input> {
    input: &'input str,
}

impl<'a> Decoder<'a> {
    pub fn new(input: &'a str) -> Self {
        Decoder { input }
    }

    fn try_decode(&self, mut chars: Peekable<std::str::Chars<'_>>) -> Result<char, String> {
        if let Some(&next) = chars.peek() {
            if next == '#' {
                chars.next(); // consume '#'
                let mut num_str = String::new();

                let is_hex = if let Some(&'x') | Some(&'X') = chars.peek() {
                    chars.next();
                    true
                } else {
                    false
                };

                while let Some(&digit) = chars.peek() {
                    if (is_hex && digit.is_ascii_hexdigit()) || (!is_hex && digit.is_ascii_digit())
                    {
                        num_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if num_str.is_empty() || chars.next() != Some(';') {
                    return Err("Invalid numeric character reference".to_string());
                }
                let base = if is_hex { 16 } else { 10 };

                if let Ok(code_point) = u32::from_str_radix(&num_str, base) {
                    if let Some(ch) = char::from_u32(code_point) {
                        return Ok(ch);
                    } else {
                        return Err("Invalid Unicode code point".to_string());
                    }
                } else {
                    return Err("Invalid numeric character reference".to_string());
                }
            } else {
                let mut entity_name = String::new();
                while let Some(&next_char) = chars.peek() {
                    if next_char.is_alphanumeric() || next_char == '_' || next_char == '-' {
                        entity_name.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if chars.next() != Some(';') || entity_name.is_empty() {
                    return Err("Invalid named character reference".to_string());
                }

                match entity_name.as_str() {
                    "amp" => Ok('&'),
                    "lt" => Ok('<'),
                    "gt" => Ok('>'),
                    "quot" => Ok('"'),
                    "apos" => Ok('\''),
                    "nbsp" => Ok('\u{00A0}'),     // Non-breaking space
                    "iexcl" => Ok('\u{00A1}'), // Incomplete entity, but often used as a non-breaking space
                    "cent" => Ok('\u{00A2}'),  // Cent sign
                    "pound" => Ok('\u{00A3}'), // Pound sign
                    "curren" => Ok('\u{00A4}'), // Currency sign
                    "yen" => Ok('\u{00A5}'),   // Yen sign
                    "brvbar" => Ok('\u{00A6}'), // Broken bar
                    "sect" => Ok('\u{00A7}'),  // Section sign
                    "uml" => Ok('\u{00A8}'),   // Diaeresis
                    "copy" => Ok('\u{00A9}'),  // Copyright sign
                    "ordf" => Ok('\u{00AA}'),  // Feminine ordinal indicator
                    "laquo" => Ok('\u{00AB}'), // Left-pointing double angle quotation mark
                    "not" => Ok('\u{00AC}'),   // Not sign
                    "shy" => Ok('\u{00AD}'),   // Soft hyphen
                    "reg" => Ok('\u{00AE}'),   // Registered sign
                    "macr" => Ok('\u{00AF}'),  // Macron
                    "deg" => Ok('\u{00B0}'),   // Degree sign
                    "plusmn" => Ok('\u{00B1}'), // Plus-minus sign
                    "sup2" => Ok('\u{00B2}'),  // Superscript two
                    "sup3" => Ok('\u{00B3}'),  // Superscript three
                    "acute" => Ok('\u{00B4}'), // Acute accent
                    "micro" => Ok('\u{00B5}'), // Micro sign
                    "para" => Ok('\u{00B6}'),  // Pilcrow sign
                    "cedil" => Ok('\u{00B8}'), // Cedilla
                    "sup1" => Ok('\u{00B9}'),  // Superscript one
                    "ordm" => Ok('\u{00BA}'),  // Masculine ordinal indicator
                    "raquo" => Ok('\u{00BB}'), // Right-pointing double angle quotation mark
                    "frac14" => Ok('\u{00BC}'), // Vulgar fraction one quarter
                    "frac12" => Ok('\u{00BD}'), // Vulgar fraction one half
                    "frac34" => Ok('\u{00BE}'), // Vulgar fraction three quarters
                    "iquest" => Ok('\u{00BF}'), // Inverted question mark
                    "times" => Ok('\u{00D7}'), // Multiplication sign
                    "divide" => Ok('\u{00F7}'), // Division sign
                    "forall" => Ok('\u{2200}'), // For all
                    "part" => Ok('\u{2202}'),  // Partial differential
                    "exist" => Ok('\u{2203}'), // There exists
                    "empty" => Ok('\u{2205}'), // Empty set
                    "nabla" => Ok('\u{2207}'), // Nabla
                    "isin" => Ok('\u{2208}'),  // Element of
                    "notin" => Ok('\u{2209}'), // Not an element of
                    "ni" => Ok('\u{220B}'),    // Contains as member
                    "prod" => Ok('\u{220F}'),  // N-ary product
                    "sum" => Ok('\u{2211}'),   // N-ary summation
                    "minus" => Ok('\u{2212}'), // Minus sign
                    "lowast" => Ok('\u{2217}'), // Asterisk operator
                    "radic" => Ok('\u{221A}'), // Square root
                    "prop" => Ok('\u{221D}'),  // Proportional to
                    "infin" => Ok('\u{221E}'), // Infinity
                    "ang" => Ok('\u{2220}'),   // Angle
                    "and" => Ok('\u{2227}'),   // Logical and
                    "or" => Ok('\u{2228}'),    // Logical or
                    "cap" => Ok('\u{2229}'),   // Intersection
                    "cup" => Ok('\u{222A}'),   // Union
                    "int" => Ok('\u{222B}'),   // Integral
                    "there4" => Ok('\u{2234}'), // Therefore
                    "sim" => Ok('\u{223C}'),   // Tilde operator
                    "cong" => Ok('\u{2245}'),  // Approximately equal to
                    "asymp" => Ok('\u{2248}'), // Almost equal to
                    "ne" => Ok('\u{2260}'),    // Not equal to
                    "equiv" => Ok('\u{2261}'), // Identical to
                    "le" => Ok('\u{2264}'),    // Less-than or equal to
                    "ge" => Ok('\u{2265}'),    // Greater-than or equal to
                    "sub" => Ok('\u{2282}'),   // Subset of
                    "sup" => Ok('\u{2283}'),   // Superset of
                    "nsub" => Ok('\u{2284}'),  // Not a subset of
                    "sube" => Ok('\u{2286}'),  // Subset of or equal to
                    "supe" => Ok('\u{2287}'),  // Superset of or equal to
                    "oplus" => Ok('\u{2295}'), // Circled plus
                    "otimes" => Ok('\u{2297}'), // Circled times
                    "perp" => Ok('\u{22A5}'),  // Up tack
                    "sdot" => Ok('\u{22C5}'),  // Dot operator
                    "Alpha" => Ok('\u{0391}'), // Greek capital letter Alpha
                    "Beta" => Ok('\u{0392}'),  // Greek capital letter Beta
                    "Gamma" => Ok('\u{0393}'), // Greek capital letter Gamma
                    "Delta" => Ok('\u{0394}'), // Greek capital letter Delta
                    "Epsilon" => Ok('\u{0395}'), // Greek capital letter Epsilon
                    "Zeta" => Ok('\u{0396}'),  // Greek capital letter Zeta
                    "Eta" => Ok('\u{0397}'),   // Greek capital letter Eta
                    "Theta" => Ok('\u{0398}'), // Greek capital letter Theta
                    "Iota" => Ok('\u{0399}'),  // Greek capital letter Iota
                    "Kappa" => Ok('\u{039A}'), // Greek capital letter Kappa
                    "Lambda" => Ok('\u{039B}'), // Greek capital letter Lambda
                    "Mu" => Ok('\u{039C}'),    // Greek capital letter Mu
                    "Nu" => Ok('\u{039D}'),    // Greek capital letter Nu
                    "Xi" => Ok('\u{039E}'),    // Greek capital letter Xi
                    "Omicron" => Ok('\u{039F}'), // Greek capital letter Omicron
                    "Pi" => Ok('\u{03A0}'),    // Greek capital letter Pi
                    "Rho" => Ok('\u{03A1}'),   // Greek capital letter Rho
                    "Sigma" => Ok('\u{03A3}'), // Greek capital letter Sigma
                    "Tau" => Ok('\u{03A4}'),   // Greek capital letter Tau
                    "Upsilon" => Ok('\u{03A5}'), // Greek capital letter Upsilon
                    "Phi" => Ok('\u{03A6}'),   // Greek capital letter Phi
                    "Chi" => Ok('\u{03A7}'),   // Greek capital letter Chi
                    "Psi" => Ok('\u{03A8}'),   // Greek capital letter Psi
                    "Omega" => Ok('\u{03A9}'), // Greek capital letter Omega
                    "alpha" => Ok('\u{03B1}'), // Greek small letter Alpha
                    "beta" => Ok('\u{03B2}'),  // Greek small letter Beta
                    "gamma" => Ok('\u{03B3}'), // Greek small letter Gamma
                    "delta" => Ok('\u{03B4}'), // Greek small letter Delta
                    "epsilon" => Ok('\u{03B5}'), // Greek small letter Epsilon
                    "zeta" => Ok('\u{03B6}'),  // Greek small letter Zeta
                    "eta" => Ok('\u{03B7}'),   // Greek small letter Eta
                    "theta" => Ok('\u{03B8}'), // Greek small letter Theta
                    "iota" => Ok('\u{03B9}'),  // Greek small letter Iota
                    "kappa" => Ok('\u{03BA}'), // Greek small letter Kappa
                    "lambda" => Ok('\u{03BB}'), // Greek small letter Lambda
                    "mu" => Ok('\u{03BC}'),    // Greek small letter Mu
                    "nu" => Ok('\u{03BD}'),    // Greek small letter Nu
                    "xi" => Ok('\u{03BE}'),    // Greek small letter Xi
                    "omicron" => Ok('\u{03BF}'), // Greek small letter Omicron
                    "pi" => Ok('\u{03C0}'),    // Greek small letter Pi
                    "rho" => Ok('\u{03C1}'),   // Greek small letter Rho
                    "sigmaf" => Ok('\u{03C2}'), // Greek small letter final Sigma
                    "sigma" => Ok('\u{03C3}'), // Greek small letter Sigma
                    "tau" => Ok('\u{03C4}'),   // Greek small letter Tau
                    "upsilon" => Ok('\u{03C5}'), // Greek small letter Upsilon
                    "phi" => Ok('\u{03C6}'),   // Greek small letter Phi
                    "chi" => Ok('\u{03C7}'),   // Greek small letter Chi
                    "psi" => Ok('\u{03C8}'),   // Greek small letter Psi
                    "omega" => Ok('\u{03C9}'), // Greek small letter Omega
                    "thetasym" => Ok('\u{03D1}'), // Greek small letter Theta symbol
                    "upsih" => Ok('\u{03D2}'), // Greek upsilon with hook symbol
                    "piv" => Ok('\u{03D6}'),   // Greek pi symbol
                    "OElig" => Ok('\u{0152}'), // Latin capital ligature OE
                    "oelig" => Ok('\u{0153}'), // Latin small ligature oe
                    "Scaron" => Ok('\u{0160}'), // Latin capital letter S with caron
                    "scaron" => Ok('\u{0161}'), // Latin small letter s with caron
                    "Yuml" => Ok('\u{0178}'),  // Latin capital letter Y with diaeresis
                    "fnof" => Ok('\u{0192}'),  // Latin small letter f with hook
                    "circ" => Ok('\u{02C6}'),  // Modifier letter circumflex accent
                    "tilde" => Ok('\u{02DC}'), // Small tilde
                    "ensp" => Ok('\u{2002}'),  // En space
                    "emsp" => Ok('\u{2003}'),  // Em space
                    "thinsp" => Ok('\u{2009}'), // Thin space
                    "zwnj" => Ok('\u{200C}'),  // Zero width non-joiner
                    "zwj" => Ok('\u{200D}'),   // Zero width joiner
                    "lrm" => Ok('\u{200E}'),   // Left-to-right mark
                    "rlm" => Ok('\u{200F}'),   // Right-to-left mark
                    "ndash" => Ok('\u{2013}'), // En dash
                    "mdash" => Ok('\u{2014}'), // Em dash
                    "lsquo" => Ok('\u{2018}'), // Left single quotation mark
                    "rsquo" => Ok('\u{2019}'), // Right single quotation mark
                    "sbquo" => Ok('\u{201A}'), // Single low-9 quotation mark
                    "ldquo" => Ok('\u{201C}'), // Left double quotation mark
                    "rdquo" => Ok('\u{201D}'), // Right double quotation mark
                    "bdquo" => Ok('\u{201E}'), // Double low-9 quotation mark
                    "dagger" => Ok('\u{2020}'), // Dagger
                    "Dagger" => Ok('\u{2021}'), // Double dagger
                    "bull" => Ok('\u{2022}'),  // Bullet
                    "hellip" => Ok('\u{2026}'), // Horizontal ellipsis
                    "permil" => Ok('\u{2030}'), // Per mille sign
                    "prime" => Ok('\u{2032}'), // Prime
                    "Prime" => Ok('\u{2033}'), // Double prime
                    "lsaquo" => Ok('\u{2039}'), // Single left-pointing angle quotation mark
                    "rsaquo" => Ok('\u{203A}'), // Single right-pointing angle quotation mark
                    "oline" => Ok('\u{203E}'), // Overline
                    "euro" => Ok('\u{20AC}'),  // Euro sign
                    "trade" => Ok('\u{2122}'), // Trade mark sign
                    "larr" => Ok('\u{2190}'),  // Leftwards arrow
                    "uarr" => Ok('\u{2191}'),  // Upwards arrow
                    "rarr" => Ok('\u{2192}'),  // Rightwards arrow
                    "darr" => Ok('\u{2193}'),  // Downwards arrow
                    "harr" => Ok('\u{2194}'),  // Left right arrow
                    "crarr" => Ok('\u{21B5}'), // Downwards arrow with corner leftwards
                    "lceil" => Ok('\u{21C0}'), // Leftwards double arrow
                    "rceil" => Ok('\u{21C1}'), // Rightwards double arrow
                    "lfloor" => Ok('\u{21C2}'), // Leftwards double arrow with stroke
                    "rfloor" => Ok('\u{21C3}'), // Rightwards double arrow with stroke
                    "loz" => Ok('\u{25CA}'),   // Lozenge
                    "spades" => Ok('\u{2660}'), // Black spade suit
                    "clubs" => Ok('\u{2663}'), // Black club suit
                    "hearts" => Ok('\u{2665}'), // Black heart suit
                    "diams" => Ok('\u{2666}'), // Black diamond suit
                    _ => return Err(format!("Unknown entity: &{};", entity_name)),
                }
            }
        } else {
            return Err("Unterminated character reference".to_string());
        }
    }

    pub fn decode(&self) -> Result<String, String> {
        if self.input.is_empty() {
            return Ok(String::new());
        }

        let mut output = String::new();
        let mut chars = self.input.chars().peekable();
        let mut position: usize = 0;

        while let Some(c) = chars.next() {
            position += 1;
            match c {
                '&' => {
                    let checkpoint = position;

                    match self.try_decode(chars.clone()) {
                        Ok(decoded_char) => {
                            output.push(decoded_char);
                            while let Some(&next_char) = chars.peek() {
                                if next_char == ';' {
                                    chars.next(); // consume ';'
                                    break;
                                } else {
                                    chars.next(); // consume the character
                                }
                            }
                        }
                        Err(_) => {
                            // If decoding fails, we assume it's not a valid entity and push the '&' back
                            output.push('&');
                            chars = self.input[checkpoint..].chars().peekable();
                            position = checkpoint;
                        }
                    }
                }
                _ => output.push(c),
            }
        }

        Ok(output)
    }
}
