//! This module defines the `Color` struct, which represents a color specified using functional notation (e.g., rgb(), rgba(), hsl(), hsla(), hwb(), lab(), oklab()).
//! It also defines the `ColorValue` enum, which represents a color component value that can be specified as a number or a percentage, and the `Alpha` struct,
//! which represents the alpha component of a color that can be specified as a number, a percentage, or "none".
//! The `Hue` struct represents the hue component of a color, which can be specified as an angle or a number (treated as degrees).
//!
//! <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/color_value>

use std::{ops::RangeInclusive, str::FromStr};

use crate::{
    color::{cielab::Cielab, oklab::Oklab, srgba::SRGBAColor},
    primitives::{angle::Angle, percentage::Percentage},
};

pub mod cielab;
pub mod hex;
pub mod named;
pub mod oklab;
pub mod srgba;

/// The hue component of a color can be specified as an angle (e.g., "120deg", "2.094rad", "133.33grad", "0.333turn") or as a number (e.g., "120"), which is treated as degrees.
///
/// Is represented as a floating-point number in degrees, normalized to the range [0, 360).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hue(f32);

impl Hue {
    /// Get the hue value as a floating-point number in degrees, normalized to the range [0, 360).
    pub fn value(&self) -> f32 {
        self.0.rem_euclid(360.0)
    }
}

impl From<f32> for Hue {
    fn from(value: f32) -> Self {
        Hue(value)
    }
}

impl FromStr for Hue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Ok(angle) = s.parse::<Angle>() {
            Ok(Hue::from(angle))
        } else {
            let number = s.parse::<f32>().map_err(|e| e.to_string())?;
            Ok(Hue(number))
        }
    }
}

impl From<Angle> for Hue {
    fn from(value: Angle) -> Self {
        Hue(value.to_degrees())
    }
}

/// A color component value can be specified as a number (e.g., "255") or a percentage (e.g., "100%").
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorValue {
    /// A number, which is clamped to the specified range (e.g., 0-255 for RGB components).
    Number(f32),

    /// A percentage, which is converted to a number based on the specified range (e.g., 100% would be 255 for RGB components).
    Percentage(Percentage),
}

/// Indicates how to interpret percentage values when converting them to numbers for color components.
pub enum Fraction {
    /// Treat percentage values as unsigned fractions, where 0% corresponds to the start of the range and 100% corresponds to the end of the range.
    Unsigned,

    /// Treat percentage values as signed fractions, where 0% corresponds to the middle of the range, 100% corresponds to the end of the range,
    /// and -100% corresponds to the start of the range.
    Signed,
}

impl ColorValue {
    /// Convert the ColorValue to a number based on the specified range and fraction type.
    ///
    /// For Number, the value is clamped to the range.
    /// For Percentage, the value is converted to a fraction and then linearly interpolated within the range.
    /// If the fraction type is Signed, the percentage is treated as a signed value where 0% corresponds to the start of the range,
    /// 100% corresponds to the end of the range, and -100% corresponds to the start of the range.
    ///
    /// # Example
    /// ```rust
    /// use css_style::{color::ColorValue, percentage::Percentage, color::Fraction};
    ///
    /// let percentage = ColorValue::Percentage(Percentage::new(50.0));
    /// assert_eq!(percentage.value(0.0..=255.0, Fraction::Unsigned), 127.5);
    /// assert_eq!(percentage.value(0.0..=255.0, Fraction::Signed), 191.25);
    /// ```
    pub fn value(&self, range: RangeInclusive<f32>, fraction: Fraction) -> f32 {
        match self {
            ColorValue::Number(n) => n.clamp(*range.start(), *range.end()),
            ColorValue::Percentage(p) => match fraction {
                Fraction::Unsigned => Self::lerp(p.as_fraction(), range),
                Fraction::Signed => Self::signed_lerp(p.as_fraction(), range),
            },
        }
    }

    /// Linearly interpolate a fraction (0.0 to 1.0) within the specified range.
    fn lerp(fraction: f32, range: RangeInclusive<f32>) -> f32 {
        let t = fraction.clamp(0.0, 1.0);
        *range.start() + t * (*range.end() - *range.start())
    }

    /// Linearly interpolate a signed fraction (-1.0 to 1.0) within the specified range, where 0.0 corresponds to the start of the range,
    /// 1.0 corresponds to the end of the range, and -1.0 corresponds to the start of the range.
    fn signed_lerp(fraction: f32, range: RangeInclusive<f32>) -> f32 {
        let t = (fraction.clamp(-1.0, 1.0) + 1.0) / 2.0;
        range.start() + t * (range.end() - range.start())
    }
}

impl From<f32> for ColorValue {
    fn from(value: f32) -> Self {
        ColorValue::Number(value)
    }
}

impl FromStr for ColorValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.eq_ignore_ascii_case("none") {
            Ok(ColorValue::Number(0.0))
        } else if s.ends_with('%') {
            Ok(ColorValue::Percentage(s.parse::<Percentage>()?))
        } else {
            Ok(ColorValue::Number(
                s.parse::<f32>().map_err(|e| e.to_string())?,
            ))
        }
    }
}

/// Alpha can be specified as a number (e.g., "0.5"), a percentage (e.g., "50%"), or "none" (which is treated as 1.0).
///
/// Always in the range [0.0, 1.0]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alpha(f32);

impl Alpha {
    /// Get the alpha value as a floating-point number in the range [0.0, 1.0].
    pub fn value(&self) -> f32 {
        self.0.clamp(0.0, 1.0)
    }
}

impl FromStr for Alpha {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.eq_ignore_ascii_case("none") {
            Ok(Alpha(1.0))
        } else if s.ends_with('%') {
            let percentage = s.parse::<Percentage>()?;
            Ok(Alpha::from(percentage))
        } else {
            let number = s.parse::<f32>().map_err(|e| e.to_string())?;
            Ok(Alpha(number))
        }
    }
}

impl From<Percentage> for Alpha {
    fn from(value: Percentage) -> Self {
        Self((value.as_fraction()).clamp(0.0, 1.0))
    }
}

/// Represents a color specified using functional notation, which can be in the form of srgba() functions (e.g., rgb(), rgba(), hsl(), hsla(), hwb()) or color() functions (e.g., lab(), oklab()).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FunctionColor {
    /// SRGBA functions with R, G, B components and optional alpha, or HSL/HWB functions with H, S, L/W/B components and optional alpha.
    ///
    /// ## Formats
    /// * rgb() or rgba()
    /// * hsl() or hsla()
    /// * hwb()
    Srgba(SRGBAColor),

    /// CIELAB color function with L, a, b components
    ///
    /// ## Formats
    /// * lab()
    /// * lch()
    Cielab(Cielab),

    /// Oklab color function with L, a, b components, e.g., oklab(0.5, 0.1, -0.1)
    ///
    /// ## Formats
    /// * oklab()
    /// * oklch()
    Oklab(Oklab),
}

impl FunctionColor {
    fn tokenize_color(input: &str, prefix: &str) -> Option<Vec<String>> {
        let input = input.trim();
        if input.starts_with(prefix) && input.ends_with(')') {
            let content = &input[prefix.len()..input.len() - 1];
            Some(
                content
                    .replace([',', '/'], " ")
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect(),
            )
        } else {
            None
        }
    }
}

impl FromStr for FunctionColor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(srgba) = s.parse::<SRGBAColor>() {
            return Ok(Self::Srgba(srgba));
        }

        if let Ok(cielab) = s.parse::<Cielab>() {
            return Ok(Self::Cielab(cielab));
        }

        if let Ok(oklab) = s.parse::<Oklab>() {
            return Ok(Self::Oklab(oklab));
        }

        Err(format!("'{}', Invalid functional color format", s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_color() {
        let input = "rgb(255, 0, 0)";
        let tokens = FunctionColor::tokenize_color(input, "rgb(").unwrap();
        assert_eq!(tokens, vec!["255", "0", "0"]);

        let input = "rgba(255 0 0 / 0.5)";
        let tokens = FunctionColor::tokenize_color(input, "rgba(").unwrap();
        assert_eq!(tokens, vec!["255", "0", "0", "0.5"]);

        let input = "hsl(120, 100%, 50%)";
        let tokens = FunctionColor::tokenize_color(input, "hsl(").unwrap();
        assert_eq!(tokens, vec!["120", "100%", "50%"]);
    }

    #[test]
    fn test_function_color_parsing() {
        let color = "rgb(255, 0, 0)".parse::<FunctionColor>().unwrap();
        assert!(matches!(color, FunctionColor::Srgba(_)));

        let color = "lab(50, 20, -30)".parse::<FunctionColor>().unwrap();
        assert!(matches!(color, FunctionColor::Cielab(_)));

        let color = "oklab(0.5, 0.1, -0.1)".parse::<FunctionColor>().unwrap();
        assert!(matches!(color, FunctionColor::Oklab(_)));
    }

    #[test]
    fn test_color_value_parsing() {
        let value = "50".parse::<ColorValue>().unwrap();
        assert_eq!(value, ColorValue::Number(50.0));

        let value = "50%".parse::<ColorValue>().unwrap();
        assert_eq!(value, ColorValue::Percentage(Percentage::new(50.0)));

        let value = "none".parse::<ColorValue>().unwrap();
        assert_eq!(value, ColorValue::Number(0.0));
    }

    #[test]
    fn test_color_value_range() {
        let value = ColorValue::Number(300.0);
        assert_eq!(value.value(0.0..=255.0, Fraction::Unsigned), 255.0);

        let value = ColorValue::Number(-10.0);
        assert_eq!(value.value(0.0..=255.0, Fraction::Unsigned), 0.0);

        let value = ColorValue::Percentage(Percentage::new(50.0));
        assert_eq!(value.value(0.0..=255.0, Fraction::Unsigned), 127.5);

        let value = ColorValue::Percentage(Percentage::new(-50.0));
        assert_eq!(value.value(0.0..=255.0, Fraction::Signed), 63.75);
    }
}
