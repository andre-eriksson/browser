//! A module for handling percentage values in CSS styles.
use std::str::FromStr;

use crate::color::ColorValue;

/// Percentage representation for CSS properties that accept percentage values, such as width, height, opacity, etc.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percentage {
    /// Percentage value (-100.0 to 100.0)
    value: f32,
}

impl Percentage {
    /// Create a new Percentage from a value (-100.0 to 100.0)
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(-100.0, 100.0),
        }
    }

    /// Create a Percentage from a fraction (-1.0 to 1.0)
    pub fn from_fraction(fraction: f32) -> Self {
        Self {
            value: fraction.clamp(-1.0, 1.0) * 100.0,
        }
    }

    /// Get the percentage value (0.0 to 100.0)
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Get the percentage as a fraction (0.0 to 1.0)
    pub fn as_fraction(&self) -> f32 {
        self.value / 100.0
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

impl FromStr for Percentage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.eq_ignore_ascii_case("none") {
            Ok(Self::new(0.0))
        } else if let Some(stripped) = s.strip_suffix('%')
            && let Ok(num) = stripped.trim().parse::<f32>()
        {
            Ok(Self::new(num))
        } else if let Ok(num) = s.trim().parse::<f32>() {
            Ok(Self::new(num))
        } else {
            Err(format!("Invalid percentage value: {}", s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_percentage() {
        assert_eq!("50%".parse(), Ok(Percentage::new(50.0)));
        assert_eq!("100%".parse(), Ok(Percentage::new(100.0)));
        assert_eq!("75.5%".parse(), Ok(Percentage::new(75.5)));
        assert!("invalid".parse::<Percentage>().is_err());
    }

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
