//! A module for handling percentage values in CSS styles.

use crate::{angle::Angle, color::ColorValue, length::Length};

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LengthPercentage {
    Length(Length),
    Percentage(Percentage),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnglePercentage {
    Angle(Angle),
    Percentage(Percentage),
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
