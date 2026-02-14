//! Defines the Dimension and MaxDimension types, which represent CSS dimension values (width, height, max-width, max-height) and their parsing from CSS component values.

use css_cssom::{ComponentValue, CssTokenKind};

use crate::{
    calculate::CalcExpression,
    length::LengthUnit,
    primitives::{length::Length, percentage::Percentage},
    properties::{AbsoluteContext, RelativeContext, RelativeType},
};

/// Represents a CSS dimension value (width or height), which can be a
/// length, percentage, calc expression, auto, max-content, min-content, fit-content, or stretch.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/width>
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Dimension {
    Percentage(Percentage),
    Length(Length),
    Calc(CalcExpression),
    #[default]
    Auto,
    MaxContent,
    MinContent,
    FitContent(Option<Length>),
    Stretch,
}

impl Dimension {
    /// Create a Dimension from a pixel value.
    pub fn px(value: f32) -> Self {
        Self::Length(Length::px(value))
    }

    /// Convert the Dimension to pixels, given the relative and absolute contexts. The rel_type indicates what the percentage is relative to.
    pub fn to_px(
        &self,
        rel_type: RelativeType,
        rel_ctx: &RelativeContext,
        abs_ctx: &AbsoluteContext,
    ) -> f32 {
        match self {
            Dimension::Length(l) => l.to_px(rel_ctx, abs_ctx),
            Dimension::MaxContent => 0.0,
            Dimension::MinContent => 0.0,
            Dimension::FitContent(_) => 0.0,
            Dimension::Stretch => 0.0,
            Dimension::Auto => 0.0,
            Dimension::Calc(calc) => calc.to_px(Some(rel_type), rel_ctx, abs_ctx),
            Dimension::Percentage(p) => match rel_type {
                RelativeType::FontSize => rel_ctx.parent.font_size * p.as_fraction(),
                RelativeType::ParentHeight => rel_ctx.parent.intrinsic_height * p.as_fraction(),
                RelativeType::ParentWidth => rel_ctx.parent.intrinsic_width * p.as_fraction(),
                RelativeType::RootFontSize => abs_ctx.root_font_size * p.as_fraction(),
                RelativeType::ViewportHeight => abs_ctx.viewport_height * p.as_fraction(),
                RelativeType::ViewportWidth => abs_ctx.viewport_width * p.as_fraction(),
            },
        }
    }
}

impl TryFrom<&[ComponentValue]> for Dimension {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Function(func) => {
                    if func.name.eq_ignore_ascii_case("calc") {
                        return Ok(Dimension::Calc(CalcExpression::parse(
                            func.value.as_slice(),
                        )?));
                    }
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("auto") {
                            return Ok(Dimension::Auto);
                        } else if ident.eq_ignore_ascii_case("max-content") {
                            return Ok(Dimension::MaxContent);
                        } else if ident.eq_ignore_ascii_case("min-content") {
                            return Ok(Dimension::MinContent);
                        } else if ident.eq_ignore_ascii_case("fit-content") {
                            return Ok(Dimension::FitContent(None)); // TODO: Fix?
                        } else if ident.eq_ignore_ascii_case("stretch") {
                            return Ok(Dimension::Stretch);
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| format!("Invalid length unit: {}", unit))?;
                        return Ok(Dimension::Length(Length::new(value.value as f32, len_unit)));
                    }
                    CssTokenKind::Percentage(pct) => {
                        return Ok(Dimension::Percentage(Percentage::new(pct.value as f32)));
                    }
                    CssTokenKind::Number(num) => {
                        return Ok(Dimension::Length(Length::px(num.value as f32)));
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Err("No valid Dimension found in component values".to_string())
    }
}

/// Represents a CSS max-dimension value (max-width or max-height), which can be a
/// length, percentage, calc expression, none, max-content, min-content, fit-content, or stretch.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/max-width>
#[derive(Debug, Clone, Default, PartialEq)]
pub enum MaxDimension {
    Length(Length),
    Percentage(Percentage),
    Calc(CalcExpression),
    #[default]
    None,
    MaxContent,
    MinContent,
    FitContent(Option<Length>),
    Stretch,
}

impl TryFrom<&[ComponentValue]> for MaxDimension {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Function(func) => {
                    if func.name.eq_ignore_ascii_case("calc") {
                        return Ok(MaxDimension::Calc(CalcExpression::parse(
                            func.value.as_slice(),
                        )?));
                    }
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("none") {
                            return Ok(MaxDimension::None);
                        } else if ident.eq_ignore_ascii_case("max-content") {
                            return Ok(MaxDimension::MaxContent);
                        } else if ident.eq_ignore_ascii_case("min-content") {
                            return Ok(MaxDimension::MinContent);
                        } else if ident.eq_ignore_ascii_case("fit-content") {
                            return Ok(MaxDimension::FitContent(None)); // TODO: Fix?
                        } else if ident.eq_ignore_ascii_case("stretch") {
                            return Ok(MaxDimension::Stretch);
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| format!("Invalid length unit: {}", unit))?;
                        return Ok(MaxDimension::Length(Length::new(
                            value.value as f32,
                            len_unit,
                        )));
                    }
                    CssTokenKind::Percentage(pct) => {
                        return Ok(MaxDimension::Percentage(Percentage::new(pct.value as f32)));
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Err("No valid MaxDimension found in component values".to_string())
    }
}

#[cfg(test)]
mod tests {
    use css_cssom::{CssToken, NumberType, NumericValue};

    use crate::ComputedStyle;

    use super::*;

    #[test]
    fn test_dimension_px() {
        let dim = Dimension::px(16.0);
        assert_eq!(dim, Dimension::Length(Length::new(16.0, LengthUnit::Px)));
    }

    #[test]
    fn test_dimension_to_px() {
        let rel_ctx = RelativeContext {
            parent: Box::new(ComputedStyle {
                font_size: 16.0,
                intrinsic_width: 200.0,
                ..Default::default()
            }),
        };
        let abs_ctx = AbsoluteContext {
            root_font_size: 16.0,
            viewport_width: 800.0,
            viewport_height: 600.0,
            ..Default::default()
        };

        let dim = Dimension::Percentage(Percentage::new(50.0));
        assert_eq!(
            dim.to_px(RelativeType::ParentWidth, &rel_ctx, &abs_ctx),
            100.0
        );
    }

    #[test]
    fn test_parse_dimension() {
        let tokens = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Dimension {
                value: NumericValue {
                    value: 16.0,
                    int_value: None,
                    type_flag: NumberType::Integer,
                    repr: String::new(),
                },
                unit: "px".to_string(),
            },
            position: None,
        })];
        let dim = Dimension::try_from(tokens.as_slice()).unwrap();
        assert_eq!(dim, Dimension::Length(Length::new(16.0, LengthUnit::Px)));
    }

    #[test]
    fn test_parse_percentage_dimension() {
        let tokens = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Percentage(NumericValue {
                value: 50.0,
                int_value: None,
                type_flag: NumberType::Integer,
                repr: String::new(),
            }),
            position: None,
        })];
        let dim = Dimension::try_from(tokens.as_slice()).unwrap();
        assert_eq!(dim, Dimension::Percentage(Percentage::new(50.0)));
    }

    #[test]
    fn test_parse_auto_dimension() {
        let tokens = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("auto".to_string()),
            position: None,
        })];
        let dim = Dimension::try_from(tokens.as_slice()).unwrap();
        assert_eq!(dim, Dimension::Auto);
    }

    #[test]
    fn test_parse_calc_dimension() {
        let tokens = vec![ComponentValue::Function(css_cssom::Function {
            name: "calc".to_string(),
            value: vec![
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Dimension {
                        value: NumericValue {
                            value: 100.0,
                            int_value: None,
                            type_flag: NumberType::Integer,
                            repr: String::new(),
                        },
                        unit: "px".to_string(),
                    },
                    position: None,
                }),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Whitespace,
                    position: None,
                }),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Delim('+'),
                    position: None,
                }),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Whitespace,
                    position: None,
                }),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Percentage(NumericValue {
                        value: 50.0,
                        int_value: None,
                        type_flag: NumberType::Integer,
                        repr: String::new(),
                    }),
                    position: None,
                }),
            ],
        })];
        let dim = Dimension::try_from(tokens.as_slice()).unwrap();
        assert!(matches!(dim, Dimension::Calc(_)));
    }

    #[test]
    fn test_parse_ident_dimension() {
        let tokens = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("max-content".to_string()),
            position: None,
        })];
        let dim = Dimension::try_from(tokens.as_slice()).unwrap();
        assert_eq!(dim, Dimension::MaxContent);
    }

    #[test]
    fn test_parse_invalid_dimension() {
        let tokens = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("invalid".to_string()),
            position: None,
        })];
        let dim = Dimension::try_from(tokens.as_slice());
        assert!(dim.is_err());
    }
}
