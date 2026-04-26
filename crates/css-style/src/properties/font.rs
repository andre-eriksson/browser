//! This module defines the data structures and logic for handling CSS font properties, including font weight, font family, and font size.
//! It provides functionality to parse these properties from CSS component values and to compute their effective values based on the context
//! of the element and its parent.

use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};
use css_values::{
    CSSParsable,
    calc::CalcKind,
    error::CssValueError,
    text::{AbsoluteSize, FontFamilyName, FontSize, GenericName, RelativeSize},
};

use crate::{
    RelativeType,
    properties::{AbsoluteContext, PixelRepr, RelativeContext},
};

impl PixelRepr for AbsoluteSize {
    fn to_px(
        self,
        _rel_type: Option<RelativeType>,
        _rel_ctx: Option<&RelativeContext>,
        _abs_ctx: &AbsoluteContext,
    ) -> Result<f64, String> {
        Ok(match self {
            Self::XxSmall => 9.0,
            Self::XSmall => 10.0,
            Self::Small => 13.0,
            Self::Medium => 16.0,
            Self::Large => 18.0,
            Self::XLarge => 24.0,
            Self::XxLarge => 32.0,
            Self::XxxLarge => 48.0,
        })
    }
}

impl PixelRepr for RelativeSize {
    fn to_px(
        self,
        _rel_type: Option<RelativeType>,
        rel_ctx: Option<&RelativeContext>,
        abs_ctx: &AbsoluteContext,
    ) -> Result<f64, String> {
        Ok(match self {
            Self::Smaller => rel_ctx.map_or(abs_ctx.root_font_size * 0.833, |ctx| ctx.parent.font_size * 0.833),
            Self::Larger => rel_ctx.map_or(abs_ctx.root_font_size * 1.2, |ctx| ctx.parent.font_size * 1.2),
        })
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
        Self {
            names: vec![FontFamilyName::default()],
        }
    }
}

impl FontFamily {
    /// Get the list of font family names in this `FontFamily`. The list is ordered by preference, with the most preferred font first.
    #[must_use]
    pub const fn names(&self) -> &Vec<FontFamilyName> {
        &self.names
    }

    /// Check if this `FontFamily` includes a monospace font, either as a generic family or as a specific font name.
    #[must_use]
    pub fn is_monospace(&self) -> bool {
        self.names.iter().any(|name| match name {
            FontFamilyName::Generic(generic) => generic == &GenericName::Monospace,
            FontFamilyName::Specific(specific) => specific.eq_ignore_ascii_case("monospace"),
        })
    }
}

impl CSSParsable for FontFamily {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();
        let checkpoint = stream.checkpoint();

        let mut full: Vec<FontFamilyName> = Vec::with_capacity(4);
        let mut names: Vec<String> = Vec::with_capacity(4);

        while let Some(cv) = stream.next_cv() {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
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
                    _ => {}
                }
            }
        }

        if names.is_empty() {
            if full.is_empty() {
                stream.restore(checkpoint);
                return Err(CssValueError::UnexpectedEndOfInput);
            }
        } else {
            let full_name = names.join(" ");
            if let Ok(generic) = full_name.parse() {
                full.push(FontFamilyName::Generic(generic));
            } else {
                full.push(FontFamilyName::Specific(full_name));
            }
        }

        Ok(Self { names: full })
    }
}

impl PixelRepr for FontSize {
    fn to_px(
        self,
        rel_type: Option<RelativeType>,
        rel_ctx: Option<&RelativeContext>,
        abs_ctx: &AbsoluteContext,
    ) -> Result<f64, String> {
        Ok(match self {
            Self::Absolute(abs) => abs.to_px(rel_type, rel_ctx, abs_ctx)?,
            Self::Length(len) => len.to_px(rel_type, rel_ctx, abs_ctx)?,
            Self::Percentage(pct) => {
                pct.as_fraction() * rel_ctx.map_or(abs_ctx.root_font_size, |ctx| ctx.parent.font_size)
            }
            Self::Relative(rel) => rel.to_px(rel_type, rel_ctx, abs_ctx)?,
            Self::Calc(expr) => {
                let kind = expr.into_sum().kind();

                match kind {
                    Ok(CalcKind::Length(len)) => len.to_px(rel_type, rel_ctx, abs_ctx)?,
                    Ok(CalcKind::Percentage(pct)) => {
                        pct.as_fraction() * rel_ctx.map_or(abs_ctx.root_font_size, |ctx| ctx.parent.font_size)
                    }
                    _ => FontSize::Absolute(AbsoluteSize::Medium).to_px(rel_type, rel_ctx, abs_ctx)?,
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use css_cssom::{CssToken, NumericValue};
    use css_values::{CSSParsable, quantity::Length, text::FontWeight};

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
