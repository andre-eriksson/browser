use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};

use crate::{CSSParsable, color::ColorValue, error::CssValueError};

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
    pub fn as_fraction(&self) -> f64 {
        self.0 / 100.0
    }
}

impl From<ColorValue> for Percentage {
    fn from(value: ColorValue) -> Self {
        match value {
            ColorValue::Percentage(pct) => pct,
            ColorValue::Number(num) => Self::from_fraction(num / 100.0),
        }
    }
}

impl CSSParsable for Percentage {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream
            .next_non_whitespace()
            .map_or(Err(CssValueError::UnexpectedEndOfInput), |cv| match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Percentage(numeric) => Ok(Self::new(numeric.to_f64())),
                    CssTokenKind::Number(numeric) => Ok(Self::from_fraction(numeric.to_f64() / 100.0)),
                    kind => Err(CssValueError::InvalidToken(kind.clone())),
                },
                _ => Err(CssValueError::InvalidComponentValue(cv.clone())),
            })
    }
}

/// Ratio representation for CSS properties that accept ratio values, such as aspect-ratio, etc.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ratio(f64, f64);

/// Flex representation for CSS properties that accept flex values, such as flex-grow, flex-shrink, etc.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flex(f64);

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
