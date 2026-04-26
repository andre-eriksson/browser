use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};

use crate::{
    CSSParsable,
    calc::{CalcDomain, CalcExpression, is_math_function},
    error::CssValueError,
    numeric::Percentage,
    quantity::{Angle, AngleUnit, Frequency, FrequencyUnit, Length, LengthUnit, Time, TimeUnit},
};

/// Represents the <length-percentage> type
///
/// Can be either a Length or a Percentage value. This is used for CSS properties
/// that accept both length and percentage values, such as width, height, margin,
/// padding, etc.
#[derive(Debug, Clone, PartialEq)]
pub enum LengthPercentage {
    Length(Length),
    Percentage(Percentage),
    Calc(CalcExpression),
}

impl From<Length> for LengthPercentage {
    fn from(length: Length) -> Self {
        Self::Length(length)
    }
}

impl From<Percentage> for LengthPercentage {
    fn from(percentage: Percentage) -> Self {
        Self::Percentage(percentage)
    }
}

impl TryFrom<&ComponentValue> for LengthPercentage {
    type Error = CssValueError;

    fn try_from(value: &ComponentValue) -> Result<Self, Self::Error> {
        match value {
            ComponentValue::Function(func) => {
                if is_math_function(&func.name) {
                    let calc_expr = CalcExpression::parse_math_function(&func.name, &func.value)?;
                    let domain = calc_expr.resolve_domain()?;

                    if !matches!(domain, CalcDomain::Length | CalcDomain::Percentage) {
                        return Err(CssValueError::InvalidCalcDomain {
                            expected: vec![CalcDomain::Length, CalcDomain::Percentage],
                            found: domain,
                        });
                    }

                    Ok(Self::Calc(calc_expr))
                } else {
                    Err(CssValueError::InvalidFunction(format!("Expected a math function, but got '{}'", func.name)))
                }
            }

            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Dimension { value, unit } => {
                    let unit =
                        LengthUnit::try_from(unit.as_str()).map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;

                    Ok(Self::Length(Length::new(value.to_f64(), unit)))
                }
                CssTokenKind::Percentage(numeric) => Ok(Self::Percentage(Percentage::new(numeric.to_f64()))),
                CssTokenKind::Number(numeric) => {
                    Ok(Self::Percentage(Percentage::from_fraction(numeric.to_f64() / 100.0)))
                }
                kind => Err(CssValueError::InvalidToken(kind.clone())),
            },
            cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
        }
    }
}

impl CSSParsable for LengthPercentage {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        if let Some(cv) = stream.next_non_whitespace() {
            Self::try_from(cv)
        } else {
            Err(CssValueError::UnexpectedEndOfInput)
        }
    }
}

/// Represents the <frequency-percentage> type
///
/// Can be either a Frequency or a Percentage value. This is used
/// for CSS properties that accept both frequency and percentage values,
/// such as audio properties.
#[derive(Debug, Clone, PartialEq)]
pub enum FrequencyPercentage {
    Frequency(Frequency),
    Percentage(Percentage),
    Calc(CalcExpression),
}

impl From<Frequency> for FrequencyPercentage {
    fn from(frequency: Frequency) -> Self {
        Self::Frequency(frequency)
    }
}

impl From<Percentage> for FrequencyPercentage {
    fn from(percentage: Percentage) -> Self {
        Self::Percentage(percentage)
    }
}

impl CSSParsable for FrequencyPercentage {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        if let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Function(func) => {
                    if is_math_function(&func.name) {
                        let calc_expr = CalcExpression::parse_math_function(&func.name, &func.value)?;
                        let domain = calc_expr.resolve_domain()?;

                        if !matches!(domain, CalcDomain::Frequency | CalcDomain::Percentage) {
                            return Err(CssValueError::InvalidCalcDomain {
                                expected: vec![CalcDomain::Frequency, CalcDomain::Percentage],
                                found: domain,
                            });
                        }

                        Ok(Self::Calc(calc_expr))
                    } else {
                        Err(CssValueError::InvalidFunction(format!(
                            "Expected a math function, but got '{}'",
                            func.name
                        )))
                    }
                }

                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Dimension { value, unit } => {
                        let unit = FrequencyUnit::try_from(unit.as_str())
                            .map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;

                        Ok(Self::Frequency(Frequency::new(value.to_f64(), unit)))
                    }
                    CssTokenKind::Percentage(numeric) => Ok(Self::Percentage(Percentage::new(numeric.to_f64()))),
                    CssTokenKind::Number(numeric) => {
                        Ok(Self::Percentage(Percentage::from_fraction(numeric.to_f64() / 100.0)))
                    }
                    kind => Err(CssValueError::InvalidToken(kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            }
        } else {
            Err(CssValueError::UnexpectedEndOfInput)
        }
    }
}

/// Represents the <angle-percentage> type
///
/// Can be either an Angle or a Percentage value. This is used for CSS properties
/// that accept both angle and percentage values, such as hue in HSL colors or
/// rotation in transforms.
#[derive(Debug, Clone, PartialEq)]
pub enum AnglePercentage {
    Angle(Angle),
    Percentage(Percentage),
    Calc(CalcExpression),
}

impl From<Angle> for AnglePercentage {
    fn from(angle: Angle) -> Self {
        Self::Angle(angle)
    }
}

impl From<Percentage> for AnglePercentage {
    fn from(percentage: Percentage) -> Self {
        Self::Percentage(percentage)
    }
}

impl TryFrom<&ComponentValue> for AnglePercentage {
    type Error = CssValueError;

    fn try_from(value: &ComponentValue) -> Result<Self, Self::Error> {
        match value {
            ComponentValue::Function(func) => {
                if is_math_function(&func.name) {
                    let calc_expr = CalcExpression::parse_math_function(&func.name, &func.value)?;
                    let domain = calc_expr.resolve_domain()?;

                    if !matches!(domain, CalcDomain::Angle | CalcDomain::Percentage) {
                        return Err(CssValueError::InvalidCalcDomain {
                            expected: vec![CalcDomain::Angle, CalcDomain::Percentage],
                            found: domain,
                        });
                    }

                    Ok(Self::Calc(calc_expr))
                } else {
                    Err(CssValueError::InvalidFunction(format!("Expected a math function, but got '{}'", func.name)))
                }
            }
            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Dimension { value, unit } => {
                    let unit =
                        AngleUnit::try_from(unit.as_str()).map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;

                    Ok(Self::Angle(Angle::new(value.to_f64(), unit)))
                }
                CssTokenKind::Percentage(numeric) => Ok(Self::Percentage(Percentage::new(numeric.to_f64()))),
                CssTokenKind::Number(numeric) => {
                    Ok(Self::Percentage(Percentage::from_fraction(numeric.to_f64() / 100.0)))
                }
                kind => Err(CssValueError::InvalidToken(kind.clone())),
            },
            cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
        }
    }
}

impl CSSParsable for AnglePercentage {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        if let Some(cv) = stream.next_non_whitespace() {
            Self::try_from(cv)
        } else {
            Err(CssValueError::UnexpectedEndOfInput)
        }
    }
}

/// Represents the <time-percentage> type
///
/// Can be either a Time or a Percentage value. This is used for CSS properties
/// that accept both time and percentage values, such as animation duration or delay.
#[derive(Debug, Clone, PartialEq)]
pub enum TimePercentage {
    Time(Time),
    Percentage(Percentage),
    Calc(CalcExpression),
}

impl From<Time> for TimePercentage {
    fn from(time: Time) -> Self {
        Self::Time(time)
    }
}

impl From<Percentage> for TimePercentage {
    fn from(percentage: Percentage) -> Self {
        Self::Percentage(percentage)
    }
}

impl CSSParsable for TimePercentage {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        if let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Function(func) => {
                    if is_math_function(&func.name) {
                        let calc_expr = CalcExpression::parse_math_function(&func.name, &func.value)?;
                        let domain = calc_expr.resolve_domain()?;

                        if !matches!(domain, CalcDomain::Time | CalcDomain::Percentage) {
                            return Err(CssValueError::InvalidCalcDomain {
                                expected: vec![CalcDomain::Time, CalcDomain::Percentage],
                                found: domain,
                            });
                        }

                        Ok(Self::Calc(calc_expr))
                    } else {
                        Err(CssValueError::InvalidFunction(format!(
                            "Expected a math function, but got '{}'",
                            func.name
                        )))
                    }
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Dimension { value, unit } => {
                        let unit =
                            TimeUnit::try_from(unit.as_str()).map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;

                        Ok(Self::Time(Time::new(value.to_f64(), unit)))
                    }
                    CssTokenKind::Percentage(numeric) => Ok(Self::Percentage(Percentage::new(numeric.to_f64()))),
                    CssTokenKind::Number(numeric) => {
                        Ok(Self::Percentage(Percentage::from_fraction(numeric.to_f64() / 100.0)))
                    }
                    kind => Err(CssValueError::InvalidToken(kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            }
        } else {
            Err(CssValueError::UnexpectedEndOfInput)
        }
    }
}

/// Represents a combination of an Angle value or the math "zero".
/// This is used for gradients.
#[derive(Debug, Clone, PartialEq)]
pub enum AngleZero {
    Angle(Angle),
    Calc(CalcExpression),
    Zero,
}

impl TryFrom<&ComponentValue> for AngleZero {
    type Error = CssValueError;

    fn try_from(value: &ComponentValue) -> Result<Self, Self::Error> {
        match value {
            ComponentValue::Function(func) => {
                if is_math_function(&func.name) {
                    let calc_expr = CalcExpression::parse_math_function(&func.name, &func.value)?;
                    let domain = calc_expr.resolve_domain()?;

                    if !matches!(domain, CalcDomain::Angle) {
                        return Err(CssValueError::InvalidCalcDomain {
                            expected: vec![CalcDomain::Angle],
                            found: domain,
                        });
                    }

                    Ok(Self::Calc(calc_expr))
                } else {
                    Err(CssValueError::InvalidFunction(format!("Expected a math function, but got '{}'", func.name)))
                }
            }
            ComponentValue::Token(token) => {
                if let CssTokenKind::Number(numeric) = &token.kind
                    && numeric.to_f64() == 0.0
                {
                    Ok(Self::Zero)
                } else {
                    Angle::try_from(token).map(AngleZero::Angle)
                }
            }
            cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
        }
    }
}

impl From<Angle> for AngleZero {
    fn from(angle: Angle) -> Self {
        Self::Angle(angle)
    }
}

/// Represents a combination of an Angle or Percentage value or the math "zero".
/// This is used for gradients.
#[derive(Debug, Clone, PartialEq)]
pub enum AnglePercentageZero {
    AnglePercentage(AnglePercentage),
    Zero,
}

impl From<AnglePercentage> for AnglePercentageZero {
    fn from(angle_percentage: AnglePercentage) -> Self {
        Self::AnglePercentage(angle_percentage)
    }
}

impl TryFrom<&ComponentValue> for AnglePercentageZero {
    type Error = CssValueError;

    fn try_from(value: &ComponentValue) -> Result<Self, Self::Error> {
        if let ComponentValue::Token(token) = value
            && token.kind == CssTokenKind::Number(0.into())
        {
            return Ok(Self::Zero);
        }

        AnglePercentage::try_from(value).map(AnglePercentageZero::AnglePercentage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use css_cssom::{ComponentValue, CssToken, CssTokenKind, Function};

    #[test]
    fn test_length_percentage_parsing() {
        let cvs = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Dimension {
                value: 10.into(),
                unit: "px".to_string(),
            },
            position: None,
        })];
        let mut stream = ComponentValueStream::new(&cvs);

        let result = LengthPercentage::parse(&mut stream).unwrap();
        assert_eq!(result, LengthPercentage::Length(Length::new(10.0, LengthUnit::Px)));

        let cvs = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Percentage(50.into()),
            position: None,
        })];
        let mut stream = ComponentValueStream::new(&cvs);

        let result = LengthPercentage::parse(&mut stream).unwrap();
        assert_eq!(result, LengthPercentage::Percentage(Percentage::new(50.0)));

        let cvs = vec![ComponentValue::Function(Function {
            name: "calc".to_string(),
            value: vec![
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Dimension {
                        value: 10.into(),
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
                    kind: CssTokenKind::Percentage(50.into()),
                    position: None,
                }),
            ],
        })];
        let mut stream = ComponentValueStream::new(&cvs);

        let result = LengthPercentage::parse(&mut stream).unwrap();
        assert!(matches!(result, LengthPercentage::Calc(_)));
        assert!(
            matches!(result, LengthPercentage::Calc(expr) if matches!(expr.resolve_domain().unwrap(), CalcDomain::Length | CalcDomain::Percentage))
        );

        let cvs = vec![ComponentValue::Function(Function {
            name: "calc".to_string(),
            value: vec![
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Number(50.into()),
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
                    kind: CssTokenKind::Percentage(50.into()),
                    position: None,
                }),
            ],
        })];
        let mut stream = ComponentValueStream::new(&cvs);

        let result = LengthPercentage::parse(&mut stream);
        assert!(result.is_err());
    }
}
