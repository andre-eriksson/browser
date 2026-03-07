use crate::color::ColorValue;

/// Percentage representation for CSS properties that accept percentage values, such as width, height, opacity, etc.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percentage(
    /// Percentage value range: (-100.0 to 100.0)
    f32,
);

impl Percentage {
    /// Create a new Percentage from a value (-100.0 to 100.0)
    pub fn new(value: f32) -> Self {
        Self(value.clamp(-100.0, 100.0))
    }

    /// Create a Percentage from a fraction (-1.0 to 1.0)
    pub fn from_fraction(fraction: f32) -> Self {
        Self(fraction.clamp(-1.0, 1.0) * 100.0)
    }

    /// Get the percentage value (0.0 to 100.0)
    pub fn value(&self) -> f32 {
        self.0
    }

    /// Get the percentage as a fraction (0.0 to 1.0)
    pub fn as_fraction(&self) -> f32 {
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

/// Ratio representation for CSS properties that accept ratio values, such as aspect-ratio, etc.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ratio(f32, f32);

/// Flex representation for CSS properties that accept flex values, such as flex-grow, flex-shrink, etc.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flex(f32);

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
