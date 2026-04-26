use std::fmt::Display;

use css_cssom::{ComponentValue, ComponentValueStream, CssToken, CssTokenKind};

use crate::{
    CSSParsable,
    calc::{CalcDomain, CalcExpression, is_math_function},
    error::CssValueError,
};

/// The primitive: <number> | calc(<number>)
#[derive(Debug, Clone, PartialEq)]
pub enum NumberOrCalc {
    Number(f64),
    Calc(CalcExpression),
}

impl From<f64> for NumberOrCalc {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl CSSParsable for NumberOrCalc {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream
            .next_non_whitespace()
            .map_or(Err(CssValueError::UnexpectedEndOfInput), |cv| match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Number(numeric) => Ok(Self::Number(numeric.to_f64())),
                    kind => Err(CssValueError::InvalidToken(kind.clone())),
                },
                ComponentValue::Function(func) => {
                    if is_math_function(&func.name) {
                        let expr = CalcExpression::parse_math_function(&func.name, &func.value)?;
                        let domain = expr.resolve_domain()?;

                        if !matches!(domain, CalcDomain::Number) {
                            return Err(CssValueError::InvalidCalcDomain {
                                expected: vec![CalcDomain::Number],
                                found: domain,
                            });
                        }

                        Ok(Self::Calc(expr))
                    } else {
                        Err(CssValueError::InvalidFunction(func.name.clone()))
                    }
                }
                _ => Err(CssValueError::InvalidComponentValue(cv.clone())),
            })
    }
}

/// Percentage representation for CSS properties that accept percentage values, such as width, height, opacity, etc.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percentage(
    /// Percentage value range: (-100.0 to 100.0)
    f64,
);

impl Percentage {
    /// Create a new Percentage from a value (-100.0 to 100.0)
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(-100.0, 100.0))
    }

    /// Create a Percentage from a fraction (-1.0 to 1.0)
    #[must_use]
    pub fn from_fraction(fraction: f64) -> Self {
        Self(fraction.clamp(-1.0, 1.0) * 100.0)
    }

    /// Get the percentage value (0.0 to 100.0)
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.0
    }

    /// Get the percentage as a fraction (0.0 to 1.0)
    #[must_use]
    pub const fn as_fraction(&self) -> f64 {
        self.0 / 100.0
    }
}

impl Display for Percentage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%", self.0)
    }
}

impl TryFrom<&CssToken> for Percentage {
    type Error = CssValueError;

    fn try_from(token: &CssToken) -> Result<Self, Self::Error> {
        match &token.kind {
            CssTokenKind::Percentage(numeric) => Ok(Self::new(numeric.to_f64())),
            CssTokenKind::Number(numeric) => Ok(Self::from_fraction(numeric.to_f64() / 100.0)),
            kind => Err(CssValueError::InvalidToken(kind.clone())),
        }
    }
}

impl CSSParsable for Percentage {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream
            .next_non_whitespace()
            .map_or(Err(CssValueError::UnexpectedEndOfInput), |cv| match cv {
                ComponentValue::Token(token) => Self::try_from(token),
                _ => Err(CssValueError::InvalidComponentValue(cv.clone())),
            })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RatioValue(pub NumberOrCalc);

impl CSSParsable for RatioValue {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        NumberOrCalc::parse(stream).map(Self)
    }
}

/// Ratio representation for CSS properties that accept ratio values, such as aspect-ratio, etc.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/ratio>
#[derive(Debug, Clone, PartialEq)]
pub struct Ratio {
    pub numerator: RatioValue,
    pub denominator: RatioValue,
}

impl CSSParsable for Ratio {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        let numerator = RatioValue::parse(stream)?;
        let denominator = if stream.next_non_whitespace().is_some() {
            RatioValue::parse(stream)?
        } else {
            RatioValue(NumberOrCalc::Number(1.0))
        };
        Ok(Self {
            numerator,
            denominator,
        })
    }
}

/// Flex representation for CSS properties that accept flex values, such as flex-grow, flex-shrink, etc.
#[derive(Debug, Clone, PartialEq)]
pub struct Flex(pub NumberOrCalc);

impl CSSParsable for Flex {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        NumberOrCalc::parse(stream).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentage_clamping() {
        assert_eq!(Percentage::new(150.0).value(), 100.0);
        assert_eq!(Percentage::new(-150.0).value(), -100.0);
    }

    #[test]
    fn test_percentage_from_fraction() {
        assert_eq!(Percentage::from_fraction(0.5).value(), 50.0);
        assert_eq!(Percentage::from_fraction(-0.5).value(), -50.0);
        assert_eq!(Percentage::from_fraction(1.5).value(), 100.0);
        assert_eq!(Percentage::from_fraction(-1.5).value(), -100.0);
    }
}
