//! This module defines the data structures and logic for handling CSS font properties, including font weight, font family, and font size.
//! It provides functionality to parse these properties from CSS component values and to compute their effective values based on the context
//! of the element and its parent.

use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};

use crate::{
    ComputedStyle, RelativeType,
    functions::math::{MathExpression, is_math_function},
    length::LengthUnit,
    primitives::{
        font::{AbsoluteSize, GenericName, RelativeSize},
        length::Length,
        percentage::Percentage,
    },
    properties::{AbsoluteContext, CSSParsable, RelativeContext},
};

/// Represents the font weight property, which can be a keyword (normal, bold) or a numeric value (100-900).
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/font-weight>
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    #[default]
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

impl TryFrom<u16> for FontWeight {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            100 => Ok(FontWeight::Thin),
            200 => Ok(FontWeight::ExtraLight),
            300 => Ok(FontWeight::Light),
            400 => Ok(FontWeight::Normal),
            500 => Ok(FontWeight::Medium),
            600 => Ok(FontWeight::SemiBold),
            700 => Ok(FontWeight::Bold),
            800 => Ok(FontWeight::ExtraBold),
            900 => Ok(FontWeight::Black),
            // TODO: Once we support variable font weights, we need to clamp between 1 and 1000 and drop the rounding logic.
            _ => Self::try_from(((value.saturating_add(50)) / 100 * 100).clamp(100, 900)),
        }
    }
}

impl CSSParsable for FontWeight {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();

        if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("normal") {
                            Ok(Self::Normal)
                        } else if ident.eq_ignore_ascii_case("bold") {
                            Ok(Self::Bold)
                        } else {
                            Err(format!("Invalid font weight keyword: {}", ident))
                        }
                    }
                    CssTokenKind::Number(num) => Self::try_from(num.to_f64() as u16),
                    _ => Err("Expected a valid font weight value".to_string()),
                },
                _ => Err("Expected a valid font weight value".to_string()),
            }
        } else {
            Err("No font weight value found".to_string())
        }
    }
}

/// Represents a font family name, which can be either a generic family (serif, sans-serif, etc.) or a specific font name.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/font-family>
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontFamilyName {
    Generic(GenericName),
    Specific(String),
}

impl Default for FontFamilyName {
    fn default() -> Self {
        FontFamilyName::Generic(GenericName::Serif)
    }
}

/// Represents the font-family property, which is a prioritized list of font family names. The browser will use the first available font from the list.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/font-family>
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontFamily {
    names: Vec<FontFamilyName>,
}

impl Default for FontFamily {
    fn default() -> Self {
        FontFamily {
            names: vec![FontFamilyName::default()],
        }
    }
}

impl FontFamily {
    /// Create a new FontFamily with a list of font family names. The list should be ordered by preference, with the most preferred font first.
    pub(crate) fn new(names: &[FontFamilyName]) -> Self {
        Self {
            names: names.to_vec(),
        }
    }

    /// Get the list of font family names in this FontFamily. The list is ordered by preference, with the most preferred font first.
    pub fn names(&self) -> &Vec<FontFamilyName> {
        &self.names
    }

    /// Check if this FontFamily includes a monospace font, either as a generic family or as a specific font name.
    pub fn is_monospace(&self) -> bool {
        self.names.iter().any(|name| match name {
            FontFamilyName::Generic(generic) => generic == &GenericName::Monospace,
            FontFamilyName::Specific(specific) => specific.eq_ignore_ascii_case("monospace"),
        })
    }
}

impl CSSParsable for FontFamily {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();
        let checkpoint = stream.checkpoint();

        let mut full: Vec<FontFamilyName> = Vec::with_capacity(4);
        let mut names: Vec<String> = Vec::with_capacity(4);

        while let Some(cv) = stream.next_cv() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        names.push(ident.clone());
                    }
                    CssTokenKind::String(s) => {
                        names.push(s.clone());
                    }
                    CssTokenKind::Comma => {
                        let full_name = names.join(" ");
                        if let Ok(generic) = full_name.parse() {
                            full.push(FontFamilyName::Generic(generic));
                        } else {
                            full.push(FontFamilyName::Specific(full_name));
                        }
                        names.clear();
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        if names.is_empty() {
            if full.is_empty() {
                stream.restore(checkpoint);
                return Err("No valid font family names found".to_string());
            }
        } else {
            let full_name = names.join(" ");
            if let Ok(generic) = full_name.parse() {
                full.push(FontFamilyName::Generic(generic));
            } else {
                full.push(FontFamilyName::Specific(full_name));
            }
        }

        Ok(FontFamily { names: full })
    }
}

/// Represents the font-size property, which can be specified using absolute-size keywords (e.g., small, medium),
/// relative-size keywords (e.g., larger, smaller), length units (e.g., 16px, 1.5em), percentage, or a calc() expression.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/font-size>
#[derive(Debug, Clone, PartialEq)]
pub enum FontSize {
    Absolute(AbsoluteSize),
    Relative(RelativeSize),
    Length(Length),
    Percentage(Percentage),
    Math(MathExpression),
}

impl Default for FontSize {
    fn default() -> Self {
        FontSize::Absolute(AbsoluteSize::Medium)
    }
}

impl FontSize {
    /// Create a FontSize from a length value in pixels.
    pub(crate) fn px(value: f32) -> Self {
        Self::Length(Length::px(value))
    }

    /// Convert this FontSize to an absolute length in pixels, given the context of the parent element's font size and the absolute context for resolving relative units.
    pub fn to_px(&self, abs_ctx: &AbsoluteContext, font_size_px: f32) -> f32 {
        let rel_ctx = RelativeContext {
            parent: ComputedStyle {
                font_size: font_size_px,
                ..Default::default()
            }
            .into(),
        };

        match self {
            FontSize::Absolute(abs) => abs.to_px(),
            FontSize::Length(len) => len.to_px(&rel_ctx, abs_ctx),
            FontSize::Percentage(pct) => pct.as_fraction() * rel_ctx.parent.font_size,
            FontSize::Relative(rel) => rel.to_px(rel_ctx.parent.font_size),
            FontSize::Math(calc) => calc.to_px(Some(RelativeType::FontSize), &rel_ctx, abs_ctx),
        }
    }
}

impl CSSParsable for FontSize {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();

        let font_size = if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Function(func) => {
                    if is_math_function(&func.name) {
                        Ok(Self::Math(MathExpression::parse_math_function(&func.name, func.value.as_slice())?))
                    } else {
                        Err(format!("Invalid function for font size: {}", func.name))
                    }
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if let Ok(abs_size) = ident.parse() {
                            Ok(Self::Absolute(abs_size))
                        } else if let Ok(rel_size) = ident.parse() {
                            Ok(Self::Relative(rel_size))
                        } else {
                            Err(format!("Invalid font size identifier: {}", ident))
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| format!("Invalid length unit: {}", unit))?;
                        Ok(Self::Length(Length::new(value.to_f64() as f32, len_unit)))
                    }
                    CssTokenKind::Percentage(num) => Ok(Self::Percentage(Percentage::new(num.to_f64() as f32))),
                    _ => Err("Expected a valid font size value".to_string()),
                },
                _ => Err("Expected a valid font size value".to_string()),
            }
        } else {
            Err("No font size value found".to_string())
        }?;

        stream.next_cv();
        Ok(font_size)
    }
}

#[cfg(test)]
mod tests {
    use css_cssom::{CssToken, NumericValue};

    use super::*;

    #[test]
    fn test_font_weight_parsing() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("bold".into()),
            position: None,
        })];
        let font_weight = FontWeight::parse(&mut input.as_slice().into()).unwrap();
        assert_eq!(font_weight, FontWeight::Bold);
    }

    #[test]
    fn test_font_family_ident_parse() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("Times".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("New".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("Roman".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("serif".to_string()),
                position: None,
            }),
        ];

        let font_family = FontFamily::parse(&mut input.as_slice().into()).unwrap();
        assert_eq!(font_family.names.len(), 2);
        assert_eq!(font_family.names[0], FontFamilyName::Specific("Times New Roman".into()));
        assert_eq!(font_family.names[1], FontFamilyName::Generic(GenericName::Serif));
    }

    #[test]
    fn test_font_family_parsing() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("Arial".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("sans-serif".into()),
                position: None,
            }),
        ];
        let font_family = FontFamily::parse(&mut input.as_slice().into()).unwrap();
        assert_eq!(font_family.names.len(), 2);
        assert_eq!(font_family.names[0], FontFamilyName::Specific("Arial".into()));
        assert_eq!(font_family.names[1], FontFamilyName::Generic(GenericName::SansSerif));
    }

    #[test]
    fn test_font_size_parsing() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Dimension {
                value: NumericValue::from(16.0),
                unit: "px".into(),
            },
            position: None,
        })];
        let font_size = FontSize::parse(&mut input.as_slice().into()).unwrap();
        assert_eq!(font_size, FontSize::Length(Length::px(16.0)));
    }
}
