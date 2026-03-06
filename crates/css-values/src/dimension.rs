use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};

use crate::{
    CSSParsable,
    calc::{CalcExpression, is_math_function},
    numeric::Percentage,
    quantity::{Length, LengthUnit},
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
}

impl CSSParsable for Dimension {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();

        if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Function(func) => {
                    if is_math_function(&func.name) {
                        Ok(Self::Calc(CalcExpression::parse_math_function(&func.name, &func.value)?))
                    } else {
                        Err(format!("Unexpected function for Dimension value: {}", func.name))
                    }
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("auto") {
                            Ok(Self::Auto)
                        } else if ident.eq_ignore_ascii_case("max-content") {
                            Ok(Self::MaxContent)
                        } else if ident.eq_ignore_ascii_case("min-content") {
                            Ok(Self::MinContent)
                        } else if ident.eq_ignore_ascii_case("fit-content") {
                            Ok(Self::FitContent(None)) // TODO: Fix?
                        } else if ident.eq_ignore_ascii_case("stretch") {
                            Ok(Self::Stretch)
                        } else {
                            Err(format!("Unexpected identifier for Dimension value: {}", ident))
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| format!("Invalid length unit: {}", unit))?;
                        Ok(Self::Length(Length::new(value.to_f64() as f32, len_unit)))
                    }
                    CssTokenKind::Number(num) => Ok(Self::Length(Length::px(num.to_f64() as f32))),
                    CssTokenKind::Percentage(pct) => Ok(Self::Percentage(Percentage::new(pct.to_f64() as f32))),
                    _ => Err(format!("Unexpected token kind for Dimension: {:?}", token.kind)),
                },
                _ => Err("Expected a token or function for Dimension value".to_string()),
            }
        } else {
            Err("Unexpected end of input while parsing Dimension value".to_string())
        }
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

impl CSSParsable for MaxDimension {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();

        if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Function(func) => {
                    if is_math_function(&func.name) {
                        Ok(MaxDimension::Calc(CalcExpression::parse_math_function(&func.name, func.value.as_slice())?))
                    } else {
                        Err(format!("Unexpected function for MaxDimension value: {}", func.name))
                    }
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("none") {
                            Ok(MaxDimension::None)
                        } else if ident.eq_ignore_ascii_case("max-content") {
                            Ok(MaxDimension::MaxContent)
                        } else if ident.eq_ignore_ascii_case("min-content") {
                            Ok(MaxDimension::MinContent)
                        } else if ident.eq_ignore_ascii_case("fit-content") {
                            Ok(MaxDimension::FitContent(None)) // TODO: Fix?
                        } else if ident.eq_ignore_ascii_case("stretch") {
                            Ok(MaxDimension::Stretch)
                        } else {
                            Err(format!("Unexpected identifier for MaxDimension value: {}", ident))
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| format!("Invalid length unit: {}", unit))?;
                        Ok(MaxDimension::Length(Length::new(value.to_f64() as f32, len_unit)))
                    }
                    CssTokenKind::Number(num) => Ok(MaxDimension::Length(Length::px(num.to_f64() as f32))),
                    CssTokenKind::Percentage(pct) => Ok(MaxDimension::Percentage(Percentage::new(pct.to_f64() as f32))),
                    _ => Err(format!("Unexpected token kind for MaxDimension: {:?}", token.kind)),
                },
                _ => Err(format!("Expected a token or function for MaxDimension value, found: {:?}", cv)),
            }
        } else {
            Err("Unexpected end of input while parsing MaxDimension value".to_string())
        }
    }
}

/// Represents a CSS offset value, used for specific margin and padding values. It can be a length, percentage, calc expression, or auto.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/padding-top>
#[derive(Debug, Clone, PartialEq)]
pub enum OffsetValue {
    Percentage(Percentage),
    Length(Length),
    Calc(CalcExpression),
    Auto,
}

impl OffsetValue {
    pub fn zero() -> Self {
        Self::Length(Length::zero())
    }

    pub fn px(value: f32) -> Self {
        Self::Length(Length::px(value))
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, OffsetValue::Auto)
    }
}

impl CSSParsable for OffsetValue {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();

        if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Function(func) if is_math_function(&func.name) => {
                    Ok(Self::Calc(CalcExpression::parse_math_function(&func.name, &func.value)?))
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| format!("Invalid length unit: {}", unit))?;
                        Ok(Self::Length(Length::new(value.to_f64() as f32, len_unit)))
                    }
                    CssTokenKind::Percentage(pct) => Ok(Self::Percentage(Percentage::new(pct.to_f64() as f32))),
                    CssTokenKind::Number(num) => Ok(Self::Length(Length::px(num.to_f64() as f32))),
                    CssTokenKind::Ident(ident) if ident.eq_ignore_ascii_case("auto") => Ok(Self::Auto),
                    _ => Err("Expected a valid offset value".to_string()),
                },
                _ => Err("Expected a valid offset value".to_string()),
            }
        } else {
            Err("Expected a valid offset value".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use css_cssom::{CssToken, NumericValue};

    use super::*;

    #[test]
    fn test_dimension_px() {
        let dim = Dimension::px(16.0);
        assert_eq!(dim, Dimension::Length(Length::new(16.0, LengthUnit::Px)));
    }

    #[test]
    fn test_parse_dimension() {
        let tokens = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Dimension {
                value: NumericValue::from(16.0),
                unit: "px".to_string(),
            },
            position: None,
        })];
        let dim = Dimension::parse(&mut tokens.as_slice().into()).unwrap();
        assert_eq!(dim, Dimension::Length(Length::new(16.0, LengthUnit::Px)));
    }

    #[test]
    fn test_parse_percentage_dimension() {
        let tokens = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Percentage(NumericValue::from(50.0)),
            position: None,
        })];
        let dim = Dimension::parse(&mut tokens.as_slice().into()).unwrap();
        assert_eq!(dim, Dimension::Percentage(Percentage::new(50.0)));
    }

    #[test]
    fn test_parse_auto_dimension() {
        let tokens = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("auto".to_string()),
            position: None,
        })];
        let dim = Dimension::parse(&mut tokens.as_slice().into()).unwrap();
        assert_eq!(dim, Dimension::Auto);
    }

    /*
    #[test]
    fn test_parse_calc_dimension() {
        let tokens = vec![ComponentValue::Function(Function {
            name: "calc".to_string(),
            value: vec![
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Dimension {
                        value: NumericValue::from(100.0),
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
                    kind: CssTokenKind::Percentage(NumericValue::from(50.0)),
                    position: None,
                }),
            ],
        })];
        let dim = Dimension::parse(&mut tokens.as_slice().into()).unwrap();
        assert!(matches!(dim, Dimension::Math(_)));
    }*/

    #[test]
    fn test_parse_ident_dimension() {
        let tokens = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("max-content".to_string()),
            position: None,
        })];
        let dim = Dimension::parse(&mut tokens.as_slice().into()).unwrap();
        assert_eq!(dim, Dimension::MaxContent);
    }

    #[test]
    fn test_parse_invalid_dimension() {
        let tokens = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("invalid".to_string()),
            position: None,
        })];
        let dim = Dimension::parse(&mut tokens.as_slice().into());
        assert!(dim.is_err());
    }
}
