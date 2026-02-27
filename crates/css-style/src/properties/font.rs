//! This module defines the data structures and logic for handling CSS font properties, including font weight, font family, and font size.
//! It provides functionality to parse these properties from CSS component values and to compute their effective values based on the context
//! of the element and its parent.

use css_cssom::{ComponentValue, CssTokenKind};

use crate::{
    ComputedStyle, RelativeType,
    functions::calculate::CalcExpression,
    length::LengthUnit,
    primitives::{
        font::{AbsoluteSize, GenericName, RelativeSize},
        length::Length,
        percentage::Percentage,
    },
    properties::{AbsoluteContext, RelativeContext},
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

impl TryFrom<&[ComponentValue]> for FontWeight {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("normal") {
                            return Ok(FontWeight::Normal);
                        } else if ident.eq_ignore_ascii_case("bold") {
                            return Ok(FontWeight::Bold);
                        }
                    }
                    CssTokenKind::Number(num) => {
                        if let Ok(weight) = FontWeight::try_from(num.to_f64() as u16) {
                            return Ok(weight);
                        }
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Err(format!("Invalid font weight value: {:?}", value))
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

impl TryFrom<&[ComponentValue]> for FontFamily {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let mut full: Vec<FontFamilyName> = Vec::with_capacity(4);
        let mut names: Vec<String> = Vec::with_capacity(4);

        for cv in value {
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
    Calc(CalcExpression),
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
            FontSize::Calc(calc) => calc.to_px(Some(RelativeType::FontSize), &rel_ctx, abs_ctx),
        }
    }
}

impl TryFrom<&[ComponentValue]> for FontSize {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Function(func) => {
                    if func.name.eq_ignore_ascii_case("calc") {
                        return Ok(FontSize::Calc(CalcExpression::parse(
                            func.value.as_slice(),
                        )?));
                    }
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if let Ok(abs_size) = ident.parse() {
                            return Ok(FontSize::Absolute(abs_size));
                        } else if let Ok(rel_size) = ident.parse() {
                            return Ok(FontSize::Relative(rel_size));
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| format!("Invalid length unit: {}", unit))?;
                        return Ok(FontSize::Length(Length::new(
                            value.to_f64() as f32,
                            len_unit,
                        )));
                    }
                    CssTokenKind::Percentage(num) => {
                        return Ok(FontSize::Percentage(Percentage::new(num.to_f64() as f32)));
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Err(format!("Invalid font size value: {:?}", value))
    }
}

#[cfg(test)]
mod tests {
    use css_cssom::CssToken;

    use crate::primitives::font::GenericName;

    use super::*;

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

        let font_family = FontFamily::try_from(input.as_slice()).unwrap();
        assert_eq!(font_family.names().len(), 2);
        assert_eq!(
            font_family.names()[0],
            FontFamilyName::Specific("Times New Roman".to_string())
        );
        assert_eq!(
            font_family.names()[1],
            FontFamilyName::Generic(GenericName::Serif)
        );
    }

    #[test]
    fn test_font_family_string_parse() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::String("Open Sans".to_string()),
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

        let font_family = FontFamily::try_from(input.as_slice()).unwrap();
        assert_eq!(font_family.names().len(), 2);
        assert_eq!(
            font_family.names()[0],
            FontFamilyName::Specific("Open Sans".to_string())
        );
        assert_eq!(
            font_family.names()[1],
            FontFamilyName::Generic(GenericName::Serif)
        );
    }
}
