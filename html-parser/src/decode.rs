pub struct Decoder<'input> {
    input: &'input String,
}

impl<'a> Decoder<'a> {
    pub fn new(input: &'a String) -> Self {
        Decoder { input }
    }

    pub fn decode(&self) -> Result<String, String> {
        let mut output = String::new();
        let mut chars = self.input.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '&' => {
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
                                if (is_hex && digit.is_ascii_hexdigit())
                                    || (!is_hex && digit.is_ascii_digit())
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
                                    output.push(ch);
                                } else {
                                    return Err("Invalid Unicode code point".to_string());
                                }
                            } else {
                                return Err("Invalid numeric character reference".to_string());
                            }
                        } else {
                            let mut entity_name = String::new();
                            while let Some(&next_char) = chars.peek() {
                                if next_char.is_alphanumeric()
                                    || next_char == '_'
                                    || next_char == '-'
                                {
                                    entity_name.push(chars.next().unwrap());
                                } else {
                                    break;
                                }
                            }
                            if chars.next() != Some(';') || entity_name.is_empty() {
                                return Err("Invalid named character reference".to_string());
                            }
                            match entity_name.as_str() {
                                "amp" => output.push('&'),
                                "lt" => output.push('<'),
                                "gt" => output.push('>'),
                                "quot" => output.push('"'),
                                "apos" => output.push('\''),
                                "nbsp" => output.push('\u{00A0}'), // Non-breaking space
                                "iexcl" => output.push('\u{00A0}'), // Incomplete entity, but often used as a non-breaking space
                                "cent" => output.push('\u{00A2}'),  // Cent sign
                                "pound" => output.push('\u{00A3}'), // Pound sign
                                "curren" => output.push('\u{00A4}'), // Currency sign
                                "yen" => output.push('\u{00A5}'),   // Yen sign
                                "brvbar" => output.push('\u{00A6}'), // Broken bar
                                "sect" => output.push('\u{00A7}'),  // Section sign
                                "uml" => output.push('\u{00A8}'),   // Diaeresis
                                "copy" => output.push('\u{00A9}'),  // Copyright sign
                                "ordf" => output.push('\u{00AA}'),  // Feminine ordinal indicator
                                "laquo" => output.push('\u{00AB}'), // Left-pointing double angle quotation mark
                                "not" => output.push('\u{00AC}'),   // Not sign
                                "shy" => output.push('\u{00AD}'),   // Soft hyphen
                                "reg" => output.push('\u{00AE}'),   // Registered sign
                                "macr" => output.push('\u{00AF}'),  // Macron
                                "deg" => output.push('\u{00B0}'),   // Degree sign
                                "plusmn" => output.push('\u{00B1}'), // Plus-minus sign
                                "sup2" => output.push('\u{00B2}'),  // Superscript two
                                "sup3" => output.push('\u{00B3}'),  // Superscript three
                                "acute" => output.push('\u{00B4}'), // Acute accent
                                "micro" => output.push('\u{00B5}'), // Micro sign
                                "para" => output.push('\u{00B6}'),  // Pilcrow sign
                                "cedil" => output.push('\u{00B8}'), // Cedilla
                                "sup1" => output.push('\u{00B9}'),  // Superscript one
                                "ordm" => output.push('\u{00BA}'),  // Masculine ordinal indicator
                                "raquo" => output.push('\u{00BB}'), // Right-pointing double angle quotation mark
                                "frac14" => output.push('\u{00BC}'), // Vulgar fraction one quarter
                                "frac12" => output.push('\u{00BD}'), // Vulgar fraction one half
                                "frac34" => output.push('\u{00BE}'), // Vulgar fraction three quarters
                                "iquest" => output.push('\u{00BF}'), // Inverted question mark
                                "times" => output.push('\u{00D7}'),  // Multiplication sign
                                "divide" => output.push('\u{00F7}'), // Division sign
                                "forall" => output.push('\u{2200}'), // For all
                                "part" => output.push('\u{2202}'),   // Partial differential
                                "exist" => output.push('\u{2203}'),  // There exists
                                "empty" => output.push('\u{2205}'),  // Empty set
                                "nabla" => output.push('\u{2207}'),  // Nabla
                                "isin" => output.push('\u{2208}'),   // Element of
                                "notin" => output.push('\u{2209}'),  // Not an element of
                                "ni" => output.push('\u{220B}'),     // Contains as member
                                "prod" => output.push('\u{220F}'),   // N-ary product
                                "sum" => output.push('\u{2211}'),    // N-ary summation
                                "minus" => output.push('\u{2212}'),  // Minus sign
                                "lowast" => output.push('\u{2217}'), // Asterisk operator
                                "radic" => output.push('\u{221A}'),  // Square root
                                "prop" => output.push('\u{221D}'),   // Proportional to
                                "infin" => output.push('\u{221E}'),  // Infinity
                                "ang" => output.push('\u{2220}'),    // Angle
                                "and" => output.push('\u{2227}'),    // Logical and
                                "or" => output.push('\u{2228}'),     // Logical or
                                "cap" => output.push('\u{2229}'),    // Intersection
                                "cup" => output.push('\u{222A}'),    // Union
                                "int" => output.push('\u{222B}'),    // Integral
                                "there4" => output.push('\u{2234}'), // Therefore
                                "sim" => output.push('\u{223C}'),    // Tilde operator
                                "cong" => output.push('\u{2245}'),   // Approximately equal to
                                "asymp" => output.push('\u{2248}'),  // Almost equal to
                                "ne" => output.push('\u{2260}'),     // Not equal to
                                "equiv" => output.push('\u{2261}'),  // Identical to
                                "le" => output.push('\u{2264}'),     // Less-than or equal to
                                "ge" => output.push('\u{2265}'),     // Greater-than or equal to
                                "sub" => output.push('\u{2282}'),    // Subset of
                                "sup" => output.push('\u{2283}'),    // Superset of
                                "nsub" => output.push('\u{2284}'),   // Not a subset of
                                "sube" => output.push('\u{2286}'),   // Subset of or equal to
                                "supe" => output.push('\u{2287}'),   // Superset of or equal to
                                "oplus" => output.push('\u{2295}'),  // Circled plus
                                "otimes" => output.push('\u{2297}'), // Circled times
                                "perp" => output.push('\u{22A5}'),   // Up tack
                                "sdot" => output.push('\u{22C5}'),   // Dot operator
                                "Alpha" => output.push('\u{0391}'),  // Greek capital letter Alpha
                                "Beta" => output.push('\u{0392}'),   // Greek capital letter Beta
                                "Gamma" => output.push('\u{0393}'),  // Greek capital letter Gamma
                                "Delta" => output.push('\u{0394}'),  // Greek capital letter Delta
                                "Epsilon" => output.push('\u{0395}'), // Greek capital letter Epsilon
                                "Zeta" => output.push('\u{0396}'),    // Greek capital letter Zeta
                                "Eta" => output.push('\u{0397}'),     // Greek capital letter Eta
                                "Theta" => output.push('\u{0398}'),   // Greek capital letter Theta
                                "Iota" => output.push('\u{0399}'),    // Greek capital letter Iota
                                "Kappa" => output.push('\u{039A}'),   // Greek capital letter Kappa
                                "Lambda" => output.push('\u{039B}'),  // Greek capital letter Lambda
                                "Mu" => output.push('\u{039C}'),      // Greek capital letter Mu
                                "Nu" => output.push('\u{039D}'),      // Greek capital letter Nu
                                "Xi" => output.push('\u{039E}'),      // Greek capital letter Xi
                                "Omicron" => output.push('\u{039F}'), // Greek capital letter Omicron
                                "Pi" => output.push('\u{03A0}'),      // Greek capital letter Pi
                                "Rho" => output.push('\u{03A1}'),     // Greek capital letter Rho
                                "Sigma" => output.push('\u{03A3}'),   // Greek capital letter Sigma
                                "Tau" => output.push('\u{03A4}'),     // Greek capital letter Tau
                                "Upsilon" => output.push('\u{03A5}'), // Greek capital letter Upsilon
                                "Phi" => output.push('\u{03A6}'),     // Greek capital letter Phi
                                "Chi" => output.push('\u{03A7}'),     // Greek capital letter Chi
                                "Psi" => output.push('\u{03A8}'),     // Greek capital letter Psi
                                "Omega" => output.push('\u{03A9}'),   // Greek capital letter Omega
                                "alpha" => output.push('\u{03B1}'),   // Greek small letter Alpha
                                "beta" => output.push('\u{03B2}'),    // Greek small letter Beta
                                "gamma" => output.push('\u{03B3}'),   // Greek small letter Gamma
                                "delta" => output.push('\u{03B4}'),   // Greek small letter Delta
                                "epsilon" => output.push('\u{03B5}'), // Greek small letter Epsilon
                                "zeta" => output.push('\u{03B6}'),    // Greek small letter Zeta
                                "eta" => output.push('\u{03B7}'),     // Greek small letter Eta
                                "theta" => output.push('\u{03B8}'),   // Greek small letter Theta
                                "iota" => output.push('\u{03B9}'),    // Greek small letter Iota
                                "kappa" => output.push('\u{03BA}'),   // Greek small letter Kappa
                                "lambda" => output.push('\u{03BB}'),  // Greek small letter Lambda
                                "mu" => output.push('\u{03BC}'),      // Greek small letter Mu
                                "nu" => output.push('\u{03BD}'),      // Greek small letter Nu
                                "xi" => output.push('\u{03BE}'),      // Greek small letter Xi
                                "omicron" => output.push('\u{03BF}'), // Greek small letter Omicron
                                "pi" => output.push('\u{03C0}'),      // Greek small letter Pi
                                "rho" => output.push('\u{03C1}'),     // Greek small letter Rho
                                "sigmaf" => output.push('\u{03C2}'), // Greek small letter final Sigma
                                "sigma" => output.push('\u{03C3}'),  // Greek small letter Sigma
                                "tau" => output.push('\u{03C4}'),    // Greek small letter Tau
                                "upsilon" => output.push('\u{03C5}'), // Greek small letter Upsilon
                                "phi" => output.push('\u{03C6}'),    // Greek small letter Phi
                                "chi" => output.push('\u{03C7}'),    // Greek small letter Chi
                                "psi" => output.push('\u{03C8}'),    // Greek small letter Psi
                                "omega" => output.push('\u{03C9}'),  // Greek small letter Omega
                                "thetasym" => output.push('\u{03D1}'), // Greek small letter Theta symbol
                                "upsih" => output.push('\u{03D2}'), // Greek upsilon with hook symbol
                                "piv" => output.push('\u{03D6}'),   // Greek pi symbol
                                "OElig" => output.push('\u{0152}'), // Latin capital ligature OE
                                "oelig" => output.push('\u{0153}'), // Latin small ligature oe
                                "Scaron" => output.push('\u{0160}'), // Latin capital letter S with caron
                                "scaron" => output.push('\u{0161}'), // Latin small letter s with caron
                                "Yuml" => output.push('\u{0178}'), // Latin capital letter Y with diaeresis
                                "fnof" => output.push('\u{0192}'), // Latin small letter f with hook
                                "circ" => output.push('\u{02C6}'), // Modifier letter circumflex accent
                                "tilde" => output.push('\u{02DC}'), // Small tilde
                                "ensp" => output.push('\u{2002}'), // En space
                                "emsp" => output.push('\u{2003}'), // Em space
                                "thinsp" => output.push('\u{2009}'), // Thin space
                                "zwnj" => output.push('\u{200C}'), // Zero width non-joiner
                                "zwj" => output.push('\u{200D}'),  // Zero width joiner
                                "lrm" => output.push('\u{200E}'),  // Left-to-right mark
                                "rlm" => output.push('\u{200F}'),  // Right-to-left mark
                                "ndash" => output.push('\u{2013}'), // En dash
                                "mdash" => output.push('\u{2014}'), // Em dash
                                "lsquo" => output.push('\u{2018}'), // Left single quotation mark
                                "rsquo" => output.push('\u{2019}'), // Right single quotation mark
                                "sbquo" => output.push('\u{201A}'), // Single low-9 quotation mark
                                "ldquo" => output.push('\u{201C}'), // Left double quotation mark
                                "rdquo" => output.push('\u{201D}'), // Right double quotation mark
                                "bdquo" => output.push('\u{201E}'), // Double low-9 quotation mark
                                "dagger" => output.push('\u{2020}'), // Dagger
                                "Dagger" => output.push('\u{2021}'), // Double dagger
                                "bull" => output.push('\u{2022}'), // Bullet
                                "hellip" => output.push('\u{2026}'), // Horizontal ellipsis
                                "permil" => output.push('\u{2030}'), // Per mille sign
                                "prime" => output.push('\u{2032}'), // Prime
                                "Prime" => output.push('\u{2033}'), // Double prime
                                "lsaquo" => output.push('\u{2039}'), // Single left-pointing angle quotation mark
                                "rsaquo" => output.push('\u{203A}'), // Single right-pointing angle quotation mark
                                "oline" => output.push('\u{203E}'),  // Overline
                                "euro" => output.push('\u{20AC}'),   // Euro sign
                                "trade" => output.push('\u{2122}'),  // Trade mark sign
                                "larr" => output.push('\u{2190}'),   // Leftwards arrow
                                "uarr" => output.push('\u{2191}'),   // Upwards arrow
                                "rarr" => output.push('\u{2192}'),   // Rightwards arrow
                                "darr" => output.push('\u{2193}'),   // Downwards arrow
                                "harr" => output.push('\u{2194}'),   // Left right arrow
                                "crarr" => output.push('\u{21B5}'), // Downwards arrow with corner leftwards
                                "lceil" => output.push('\u{21C0}'), // Leftwards double arrow
                                "rceil" => output.push('\u{21C1}'), // Rightwards double arrow
                                "lfloor" => output.push('\u{21C2}'), // Leftwards double arrow with stroke
                                "rfloor" => output.push('\u{21C3}'), // Rightwards double arrow with stroke
                                "loz" => output.push('\u{25CA}'),    // Lozenge
                                "spades" => output.push('\u{2660}'), // Black spade suit
                                "clubs" => output.push('\u{2663}'),  // Black club suit
                                "hearts" => output.push('\u{2665}'), // Black heart suit
                                "diams" => output.push('\u{2666}'),  // Black diamond suit
                                _ => return Err(format!("Unknown entity: &{};", entity_name)),
                            }
                        }
                    } else {
                        return Err("Unterminated character reference".to_string());
                    }
                }
                _ => output.push(c),
            }
        }

        Ok(output)
    }
}
