use css_cssom::{CssToken, CssTokenKind};

use crate::color::{function::ColorFunction, named::NamedColor};

/// Represents a color specified in hexadecimal format, supporting 3, 4, 6, or 8 digit formats.
///
/// The RGB components are stored as u8 values (0 to 255), and the alpha component is also stored
/// as a u8 value (0 to 255, where 255 is fully opaque).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl TryFrom<&CssToken> for HexColor {
    type Error = String;

    fn try_from(value: &CssToken) -> Result<Self, Self::Error> {
        if let CssTokenKind::Hash { value, .. } = &value.kind {
            let parsed = u32::from_str_radix(value, 16).map_err(|e| e.to_string())?;

            match value.len() {
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
                _ => Err(format!("'{value}', Invalid hex color format")),
            }
        } else {
            Err("Expected a hash token for hex color".to_string())
        }
    }
}

/// Represents the <color-base> type in CSS, including hexadecimal colors, functional colors (like `rgb()`, `hsl()`), named colors, and the transparent keyword.
#[derive(Debug, Clone, PartialEq)]
pub enum ColorBase {
    Hex(HexColor),
    Function(ColorFunction),
    Named(NamedColor),

    /// The 'transparent' keyword represents a fully transparent color, which is equivalent to rgba(0, 0, 0, 0).
    /// It is a special case in CSS and does not have a specific RGB value, but it can be treated as having an
    /// alpha value of 0 for rendering purposes.
    Transparent,
    // TODO: color-mix()
}

#[cfg(test)]
mod tests {
    use css_cssom::CSSStyleSheet;

    use super::*;

    #[test]
    fn test_hex_parsing_three() {
        let css = CSSStyleSheet::from_inline("color: #0F3");
        let hex = HexColor::try_from(css[0].original_values[0].as_token().unwrap()).unwrap();

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
        let css = CSSStyleSheet::from_inline("color: #0F3A");
        let hex = HexColor::try_from(css[0].original_values[0].as_token().unwrap()).unwrap();

        assert_eq!(
            hex,
            HexColor {
                r: 0,
                g: 255,
                b: 51,
                a: 170
            }
        );
    }

    #[test]
    fn test_hex_parsing_six() {
        let css = CSSStyleSheet::from_inline("color: #00FF33");
        let hex = HexColor::try_from(css[0].original_values[0].as_token().unwrap()).unwrap();

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
    fn test_hex_parsing_eight() {
        let css = CSSStyleSheet::from_inline("color: #00FF33AA");
        let hex = HexColor::try_from(css[0].original_values[0].as_token().unwrap()).unwrap();

        assert_eq!(
            hex,
            HexColor {
                r: 0,
                g: 255,
                b: 51,
                a: 170
            }
        );
    }
}
