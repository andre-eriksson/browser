//! This module defines the `Color` struct, which represents a color specified using functional notation (e.g., rgb(), rgba(), hsl(), hsla(), hwb(), lab(), oklab()).
//! It also defines the `ColorValue` enum, which represents a color component value that can be specified as a number or a percentage, and the `Alpha` struct,
//! which represents the alpha component of a color that can be specified as a number, a percentage, or "none".
//! The `Hue` struct represents the hue component of a color, which can be specified as an angle or a number (treated as degrees).
//!
//! <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/color_value>

use std::ops::RangeInclusive;

use css_cssom::{ComponentValue, CssTokenKind};

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

impl From<ColorValue> for Hue {
    fn from(value: ColorValue) -> Self {
        match value {
            ColorValue::Number(n) => Hue(n),
            ColorValue::Percentage(p) => Hue(p.as_fraction() * 360.0),
        }
    }
}

impl From<f32> for Hue {
    fn from(value: f32) -> Self {
        Hue(value)
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

impl TryFrom<&[ComponentValue]> for ColorValue {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Whitespace => continue,
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("none") {
                            return Ok(ColorValue::Number(0.0));
                        } else {
                            return Err(format!("Invalid color value: '{}'", ident));
                        }
                    }
                    CssTokenKind::Percentage(percent) => {
                        return Ok(ColorValue::Percentage(Percentage::new(
                            percent.value as f32,
                        )));
                    }
                    CssTokenKind::Number(num) => {
                        return Ok(ColorValue::Number(num.value as f32));
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Err("No valid color value found".to_string())
    }
}

impl From<f32> for ColorValue {
    fn from(value: f32) -> Self {
        ColorValue::Number(value)
    }
}

/// Alpha can be specified as a number (e.g., "0.5"), a percentage (e.g., "50%"), or "none" (which is treated as 1.0).
///
/// Always in the range [0.0, 1.0]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alpha(f32);

impl Alpha {
    /// Create a new Alpha value from a floating-point number, which is clamped to the range [0.0, 1.0].
    pub fn new(value: f32) -> Self {
        Alpha(value.clamp(0.0, 1.0))
    }

    /// Get the alpha value as a floating-point number in the range [0.0, 1.0].
    pub fn value(&self) -> f32 {
        self.0.clamp(0.0, 1.0)
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

struct RawColorComponents {
    channels: [Option<ColorValue>; 3],
    alpha: Alpha,
}

impl FunctionColor {
    // TODO: Relative color syntax `color-function(from <origin> channel1 channel2 channel3)`

    fn parse_color_components(values: &[ComponentValue]) -> Result<RawColorComponents, String> {
        let mut channels = [None, None, None];
        let mut channel_idx = 0;
        let mut alpha = Alpha(1.0);
        let mut parsing_alpha = false;

        for arg in values {
            match arg {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Whitespace => continue,
                    CssTokenKind::Comma => continue,
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("none") {
                            if parsing_alpha {
                                alpha = Alpha(1.0);
                            } else if channel_idx < 3 {
                                channels[channel_idx] = Some(ColorValue::Number(0.0));
                                channel_idx += 1;
                            } else {
                                return Err("Too many components in color function".to_string());
                            }
                        } else {
                            return Err(format!("Invalid token in color function: '{}'", ident));
                        }
                    }
                    CssTokenKind::Delim('/') => {
                        parsing_alpha = true;
                    }
                    CssTokenKind::Percentage(pct) => {
                        if parsing_alpha {
                            alpha = Alpha::from(Percentage::new(pct.value as f32));
                        } else if channel_idx < 3 {
                            channels[channel_idx] =
                                Some(ColorValue::Percentage(Percentage::new(pct.value as f32)));
                            channel_idx += 1;
                        } else {
                            return Err("Too many components in color function".to_string());
                        }
                    }
                    CssTokenKind::Number(num) => {
                        if parsing_alpha {
                            alpha = Alpha::new(num.value as f32);
                        } else if channel_idx < 3 {
                            channels[channel_idx] = Some(ColorValue::Number(num.value as f32));
                            channel_idx += 1;
                        } else {
                            return Err("Too many components in color function".to_string());
                        }
                    }
                    _ => return Err(format!("Invalid token in color function: {:?}", token)),
                },
                _ => return Err("Unsupported component value in color function".to_string()),
            }
        }

        Ok(RawColorComponents { channels, alpha })
    }
}

impl TryFrom<&[ComponentValue]> for FunctionColor {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        if let Ok(srgba) = value.try_into() {
            return Ok(Self::Srgba(srgba));
        }

        if let Ok(cielab) = value.try_into() {
            return Ok(Self::Cielab(cielab));
        }

        if let Ok(oklab) = value.try_into() {
            return Ok(Self::Oklab(oklab));
        }

        Err("Invalid functional color format".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
