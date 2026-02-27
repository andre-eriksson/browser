//! Hexadecimal color notation (e.g., #RRGGBB, #RGB, #RRGGBBAA, #RGBA)

use css_cssom::{ComponentValue, CssTokenKind};

/// Hex color representations as defined in CSS Color Module Level 4
///
/// It is parsed via the hex formats:
/// * #RRGGBB (6 hex digits)
/// * #RRGGBBAA (8 hex digits)
/// * #RGB (3 hex digits, where each digit is repeated to form the full value)
/// * #RGBA (4 hex digits, where each digit is repeated to form the full value, and the last digit represents alpha)
///
/// However it is stored as separate RGBA components for easier manipulation and conversion to other color formats.
/// The alpha component is optional and defaults to 255 (fully opaque) if not provided.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HexColor {
    /// R component (0 to 255)
    pub r: u8,

    /// G component (0 to 255)
    pub g: u8,

    /// B component (0 to 255)
    pub b: u8,

    /// A component (0 to 255, where 255 is fully opaque)
    pub a: u8,
}

impl TryFrom<&[ComponentValue]> for HexColor {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Hash { value, .. } => {
                        let parsed = u32::from_str_radix(value, 16).map_err(|e| e.to_string())?;

                        return match value.len() {
                            3 | 4 => {
                                let r = ((parsed >> (if value.len() == 4 { 12 } else { 8 })) & 0xF) as u8;
                                let g = ((parsed >> (if value.len() == 4 { 8 } else { 4 })) & 0xF) as u8;
                                let b = ((parsed >> (if value.len() == 4 { 4 } else { 0 })) & 0xF) as u8;
                                let a = if value.len() == 4 {
                                    (parsed & 0xF) as u8
                                } else {
                                    15
                                };
                                Ok(Self {
                                    r: r * 17,
                                    g: g * 17,
                                    b: b * 17,
                                    a: a * 17,
                                })
                            }
                            6 => {
                                let r = ((parsed >> 16) & 0xFF) as u8;
                                let g = ((parsed >> 8) & 0xFF) as u8;
                                let b = (parsed & 0xFF) as u8;
                                Ok(Self { r, g, b, a: 255 })
                            }
                            8 => {
                                let r = ((parsed >> 24) & 0xFF) as u8;
                                let g = ((parsed >> 16) & 0xFF) as u8;
                                let b = ((parsed >> 8) & 0xFF) as u8;
                                let a = (parsed & 0xFF) as u8;
                                Ok(Self { r, g, b, a })
                            }
                            _ => Err(format!("'{}', Invalid hex color format", value)),
                        };
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Err(format!("No valid hex color token found in component values: {:?}", value))
    }
}

#[cfg(test)]
mod tests {
    use css_cssom::CssParser;

    use super::*;

    #[test]
    fn test_hex_parsing_three() {
        let mut parser = CssParser::new(None);
        let three = parser.parse_css("* { color: #0F3; } ", false);
        let color = &three.rules[0].as_qualified_rule().unwrap().block.value[4];
        let hex = HexColor::try_from(&[color.clone()][..]).unwrap();
        assert_eq!(
            hex,
            HexColor {
                r: 0,
                g: 255,
                b: 51,
                a: 255
            }
        );
    }

    #[test]
    fn test_hex_parsing_four() {
        let mut parser = CssParser::new(None);
        let four = parser.parse_css("* { color: #0F38; } ", false);
        let color = &four.rules[0].as_qualified_rule().unwrap().block.value[4];
        let hex = HexColor::try_from(&[color.clone()][..]).unwrap();
        assert_eq!(
            hex,
            HexColor {
                r: 0,
                g: 255,
                b: 51,
                a: 136
            }
        );
    }

    #[test]
    fn test_hex_parsing_six() {
        let mut parser = CssParser::new(None);
        let six = parser.parse_css("* { color: #FF0000; } ", false);
        let color = &six.rules[0].as_qualified_rule().unwrap().block.value[4];
        let hex = HexColor::try_from(&[color.clone()][..]).unwrap();
        assert_eq!(
            hex,
            HexColor {
                r: 255,
                g: 0,
                b: 0,
                a: 255
            }
        );

        let mut parser = CssParser::new(None);
        let eight = parser.parse_css("* { color: #FF000080; } ", false);
        let color = &eight.rules[0].as_qualified_rule().unwrap().block.value[4];
        let hex = HexColor::try_from(&[color.clone()][..]).unwrap();
        assert_eq!(
            hex,
            HexColor {
                r: 255,
                g: 0,
                b: 0,
                a: 128
            }
        );
    }

    #[test]
    fn test_hex_parsing_eight() {
        let mut parser = CssParser::new(None);
        let eight = parser.parse_css("* { color: #FF000080; } ", false);
        let color = &eight.rules[0].as_qualified_rule().unwrap().block.value[4];
        let hex = HexColor::try_from(&[color.clone()][..]).unwrap();
        assert_eq!(
            hex,
            HexColor {
                r: 255,
                g: 0,
                b: 0,
                a: 128
            }
        );
    }
}
