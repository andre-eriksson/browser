use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};

use crate::{
    CSSParsable,
    calc::{CalcDomain, CalcExpression, is_math_function},
    error::CssValueError,
    numeric::Percentage,
    quantity::{Length, LengthUnit},
};

/// Represents a CSS dimension value (width or height), which can be a
/// length, percentage, calc expression, auto, max-content, min-content, fit-content, or stretch.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/width>
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Size {
    Percentage(Percentage),
    Length(Length),
    Calc(CalcExpression),
    #[default]
    Auto,
    MaxContent,
    MinContent,
    FitContent,
    Stretch,
}

impl Size {
    /// Create a Dimension from a pixel value.
    #[must_use]
    pub const fn px(value: f64) -> Self {
        Self::Length(Length::px(value))
    }
}

impl TryFrom<&ComponentValue> for Size {
    type Error = CssValueError;

    fn try_from(cv: &ComponentValue) -> Result<Self, Self::Error> {
        match cv {
            ComponentValue::Function(func) => {
                if is_math_function(&func.name) {
                    let expr = CalcExpression::parse(&func.name, &func.value)?;
                    let domain = expr.resolve_domain()?;

                    if !matches!(domain, CalcDomain::Length | CalcDomain::Percentage) {
                        return Err(CssValueError::InvalidCalcDomain {
                            expected: vec![CalcDomain::Length, CalcDomain::Percentage],
                            found: domain,
                        });
                    }

                    Ok(Self::Calc(expr))
                } else {
                    Err(CssValueError::InvalidFunction(func.name.clone()))
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
                        Ok(Self::FitContent)
                    } else if ident.eq_ignore_ascii_case("stretch") {
                        Ok(Self::Stretch)
                    } else {
                        Err(CssValueError::InvalidValue(format!("Invalid identifier: {ident}")))
                    }
                }
                CssTokenKind::Dimension { value, unit } => {
                    let len_unit = unit
                        .parse::<LengthUnit>()
                        .map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;
                    Ok(Self::Length(Length::new(value.to_f64(), len_unit)))
                }
                CssTokenKind::Number(num) => Ok(Self::Length(Length::px(num.to_f64()))),
                CssTokenKind::Percentage(pct) => Ok(Self::Percentage(Percentage::new(pct.to_f64()))),
                _ => Err(CssValueError::InvalidToken(token.kind.clone())),
            },
            cvs @ ComponentValue::SimpleBlock(_) => Err(CssValueError::InvalidComponentValue(cvs.clone())),
        }
    }
}

impl CSSParsable for Size {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        if let Some(cv) = stream.next_non_whitespace() {
            Ok(Self::try_from(cv)?)
        } else {
            Err(CssValueError::ExpectedComponentValue)
        }
    }
}

/// Represents a CSS max-dimension value (max-width or max-height), which can be a
/// length, percentage, calc expression, none, max-content, min-content, fit-content, or stretch.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/max-width>
#[derive(Debug, Clone, Default, PartialEq)]
pub enum MaxSize {
    Length(Length),
    Percentage(Percentage),
    Calc(CalcExpression),
    #[default]
    None,
    MaxContent,
    MinContent,
    FitContent,
    Stretch,
}

impl MaxSize {
    /// Create a `MaxDimension` from a pixel value.
    #[must_use]
    pub const fn px(value: f64) -> Self {
        Self::Length(Length::px(value))
    }
}

impl CSSParsable for MaxSize {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        if let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Function(func) => {
                    if is_math_function(&func.name) {
                        let expr = CalcExpression::parse(&func.name, func.value.as_slice())?;
                        let domain = expr.resolve_domain()?;

                        if !matches!(domain, CalcDomain::Length | CalcDomain::Percentage) {
                            return Err(CssValueError::InvalidCalcDomain {
                                expected: vec![CalcDomain::Length, CalcDomain::Percentage],
                                found: domain,
                            });
                        }

                        Ok(Self::Calc(expr))
                    } else {
                        Err(CssValueError::InvalidFunction(func.name.clone()))
                    }
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("none") {
                            Ok(Self::None)
                        } else if ident.eq_ignore_ascii_case("max-content") {
                            Ok(Self::MaxContent)
                        } else if ident.eq_ignore_ascii_case("min-content") {
                            Ok(Self::MinContent)
                        } else if ident.eq_ignore_ascii_case("fit-content") {
                            Ok(Self::FitContent) // TODO: Fix?
                        } else if ident.eq_ignore_ascii_case("stretch") {
                            Ok(Self::Stretch)
                        } else {
                            Err(CssValueError::InvalidValue(format!("Invalid identifier: {ident}")))
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;
                        Ok(Self::Length(Length::new(value.to_f64(), len_unit)))
                    }
                    CssTokenKind::Number(num) => Ok(Self::Length(Length::px(num.to_f64()))),
                    CssTokenKind::Percentage(pct) => Ok(Self::Percentage(Percentage::new(pct.to_f64()))),
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs @ ComponentValue::SimpleBlock(_) => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            }
        } else {
            Err(CssValueError::UnexpectedEndOfInput)
        }
    }
}

/// Represents a CSS offset value, used for border & padding offsets. It can be a length, percentage, or calc expression.
#[derive(Debug, Clone, PartialEq)]
pub enum OffsetValue {
    Length(Length),
    Percentage(Percentage),
    Calc(CalcExpression),
}

impl OffsetValue {
    #[must_use]
    pub const fn zero() -> Self {
        Self::Length(Length::zero())
    }

    #[must_use]
    pub const fn px(value: f64) -> Self {
        Self::Length(Length::px(value))
    }
}

impl CSSParsable for OffsetValue {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        if let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Function(func) if is_math_function(&func.name) => {
                    let expr = CalcExpression::parse(&func.name, &func.value)?;
                    let domain = expr.resolve_domain()?;

                    if !matches!(domain, crate::calc::CalcDomain::Length | crate::calc::CalcDomain::Percentage) {
                        return Err(CssValueError::InvalidCalcDomain {
                            expected: vec![
                                crate::calc::CalcDomain::Length,
                                crate::calc::CalcDomain::Percentage,
                            ],
                            found: domain,
                        });
                    }

                    Ok(Self::Calc(expr))
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;
                        Ok(Self::Length(Length::new(value.to_f64(), len_unit)))
                    }
                    CssTokenKind::Percentage(pct) => Ok(Self::Percentage(Percentage::new(pct.to_f64()))),
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            }
        } else {
            Err(CssValueError::ExpectedComponentValue)
        }
    }
}

impl Default for OffsetValue {
    fn default() -> Self {
        Self::zero()
    }
}

/// Represents a CSS margin value, used for specific margin values. It can be a length, percentage, calc expression, or auto.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/margin-top>
#[derive(Debug, Clone, PartialEq)]
pub enum MarginValue {
    Percentage(Percentage),
    Length(Length),
    Calc(CalcExpression),
    Auto,
}

impl MarginValue {
    #[must_use]
    pub const fn zero() -> Self {
        Self::Length(Length::zero())
    }

    #[must_use]
    pub const fn px(value: f64) -> Self {
        Self::Length(Length::px(value))
    }

    #[must_use]
    pub const fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }
}

impl Default for MarginValue {
    fn default() -> Self {
        Self::zero()
    }
}

impl CSSParsable for MarginValue {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        if let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Function(func) if is_math_function(&func.name) => {
                    let expr = CalcExpression::parse(&func.name, &func.value)?;
                    let domain = expr.resolve_domain()?;

                    if !matches!(domain, crate::calc::CalcDomain::Length | crate::calc::CalcDomain::Percentage) {
                        return Err(CssValueError::InvalidCalcDomain {
                            expected: vec![
                                crate::calc::CalcDomain::Length,
                                crate::calc::CalcDomain::Percentage,
                            ],
                            found: domain,
                        });
                    }

                    Ok(Self::Calc(expr))
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;
                        Ok(Self::Length(Length::new(value.to_f64(), len_unit)))
                    }
                    CssTokenKind::Percentage(pct) => Ok(Self::Percentage(Percentage::new(pct.to_f64()))),
                    CssTokenKind::Ident(ident) if ident.eq_ignore_ascii_case("auto") => Ok(Self::Auto),
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            }
        } else {
            Err(CssValueError::ExpectedComponentValue)
        }
    }
}

#[cfg(test)]
mod tests {
    use css_cssom::{CssToken, NumericValue};

    use super::*;

    #[test]
    fn test_dimension_px() {
        let dim = Size::px(16.0);
        assert_eq!(dim, Size::Length(Length::new(16.0, LengthUnit::Px)));
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
        let dim = Size::parse(&mut tokens.as_slice().into()).unwrap();
        assert_eq!(dim, Size::Length(Length::new(16.0, LengthUnit::Px)));
    }

    #[test]
    fn test_parse_percentage_dimension() {
        let tokens = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Percentage(NumericValue::from(50.0)),
            position: None,
        })];
        let dim = Size::parse(&mut tokens.as_slice().into()).unwrap();
        assert_eq!(dim, Size::Percentage(Percentage::new(50.0)));
    }

    #[test]
    fn test_parse_auto_dimension() {
        let tokens = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("auto".to_string()),
            position: None,
        })];
        let dim = Size::parse(&mut tokens.as_slice().into()).unwrap();
        assert_eq!(dim, Size::Auto);
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
        let dim = Size::parse(&mut tokens.as_slice().into()).unwrap();
        assert_eq!(dim, Size::MaxContent);
    }

    #[test]
    fn test_parse_invalid_dimension() {
        let tokens = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("invalid".to_string()),
            position: None,
        })];
        let dim = Size::parse(&mut tokens.as_slice().into());
        assert!(dim.is_err());
    }
}
