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
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("none") {
                            return Ok(ColorValue::Number(0.0));
                        } else {
                            return Err(format!("Invalid color value: '{}'", ident));
                        }
                    }
                    CssTokenKind::Percentage(percent) => {
                        return Ok(ColorValue::Percentage(Percentage::new(
                            percent.to_f64() as f32
                        )));
                    }
                    CssTokenKind::Number(num) => {
                        return Ok(ColorValue::Number(num.to_f64() as f32));
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct RawColorComponents {
    channels: [Option<ColorValue>; 3],
    alpha: Alpha,
}

impl FunctionColor {
    // TODO: Relative color syntax `color-function(from <origin> channel1 channel2 channel3)`

    fn parse_color_components(values: &[ComponentValue]) -> Result<RawColorComponents, String> {
        let mut channels = [None, None, None];
        let mut channel_idx = 0;
        let mut alpha = None;
        let mut parsing_alpha = false;

        for arg in values {
            match arg {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("none") {
                            if parsing_alpha {
                                alpha = Some(Alpha(1.0));
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
                            alpha = Some(Alpha::from(Percentage::new(pct.to_f64() as f32)));
                        } else if channel_idx < 3 {
                            channels[channel_idx] =
                                Some(ColorValue::Percentage(Percentage::new(pct.to_f64() as f32)));
                            channel_idx += 1;
                        } else if alpha.is_none() {
                            alpha = Some(Alpha::from(Percentage::new(pct.to_f64() as f32)));
                        } else {
                            return Err(
                                "Too many percentage components in color function".to_string()
                            );
                        }
                    }
                    CssTokenKind::Number(num) => {
                        if parsing_alpha {
                            alpha = Some(Alpha::new(num.to_f64() as f32));
                        } else if channel_idx < 3 {
                            channels[channel_idx] = Some(ColorValue::Number(num.to_f64() as f32));
                            channel_idx += 1;
                        } else if alpha.is_none() {
                            alpha = Some(Alpha::new(num.to_f64() as f32));
                        } else {
                            return Err("Too many number components in color function".to_string());
                        }
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Ok(RawColorComponents {
            channels,
            alpha: alpha.unwrap_or(Alpha(1.0)),
        })
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
pub mod macros {
    #[macro_export]
    macro_rules! css_color_fn {
        ($func:expr, $ch1:expr, $ch2:expr, $ch3:expr, $alpha:expr) => {{
            use css_cssom::{CssToken, CssTokenKind, Function, NumericValue};
            macro_rules! css_value_token {
                ($val:expr) => {{
                    let s = $val.to_string();
                    if s == "none" {
                        CssToken {
                            kind: CssTokenKind::Ident("none".to_string()),
                            position: None,
                        }
                    } else if s.contains('%') {
                        CssToken {
                            kind: CssTokenKind::Percentage(NumericValue::from(
                                s.replace('%', "").parse::<f64>().unwrap_or(0.0),
                            )),
                            position: None,
                        }
                    } else {
                        CssToken {
                            kind: CssTokenKind::Number(NumericValue::from(
                                s.parse::<f64>().unwrap_or(0.0),
                            )),
                            position: None,
                        }
                    }
                }};
            }

            let channel1_token = css_value_token!($ch1);
            let channel2_token = css_value_token!($ch2);
            let channel3_token = css_value_token!($ch3);

            let alpha_token = match $alpha {
                a if (a.to_string().eq("none")) => CssToken {
                    kind: CssTokenKind::Ident("none".to_string()),
                    position: None,
                },
                a if (a.to_string().contains('%')) => CssToken {
                    kind: CssTokenKind::Percentage(NumericValue::from(
                        a.to_string().replace('%', "").parse::<f64>().unwrap_or(0.0),
                    )),
                    position: None,
                },
                a if (0.0..=1.0).contains(&(a.to_string().parse::<f32>().unwrap_or(-1.0))) => {
                    CssToken {
                        kind: CssTokenKind::Number(NumericValue::from(
                            a.to_string().parse::<f64>().unwrap_or(0.0),
                        )),
                        position: None,
                    }
                }
                _ => panic!("Invalid alpha value for css_color_fn_alpha! macro"),
            };

            if alpha_token.kind == CssTokenKind::Ident("none".to_string()) {
                vec![ComponentValue::Function(Function {
                    name: $func.to_string(),
                    value: vec![
                        ComponentValue::Token(channel1_token),
                        ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Whitespace,
                            position: None,
                        }),
                        ComponentValue::Token(channel2_token),
                        ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Whitespace,
                            position: None,
                        }),
                        ComponentValue::Token(channel3_token),
                    ],
                })]
            } else {
                vec![ComponentValue::Function(Function {
                    name: $func.to_string(),
                    value: vec![
                        ComponentValue::Token(channel1_token),
                        ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Whitespace,
                            position: None,
                        }),
                        ComponentValue::Token(channel2_token),
                        ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Whitespace,
                            position: None,
                        }),
                        ComponentValue::Token(channel3_token),
                        ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Whitespace,
                            position: None,
                        }),
                        ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Delim('/'),
                            position: None,
                        }),
                        ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Whitespace,
                            position: None,
                        }),
                        ComponentValue::Token(alpha_token),
                    ],
                })]
            }
        }};
    }
}

#[cfg(test)]
mod tests {
    use css_cssom::{CssToken, NumericValue};

    use super::*;

    #[test]
    fn test_parse_color_components() {
        let components = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Number(NumericValue::from(255.0)),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Percentage(NumericValue::from(50.0)),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("none".to_string()),
                position: None,
            }),
        ];

        let result = FunctionColor::parse_color_components(&components).unwrap();
        assert_eq!(result.channels[0], Some(ColorValue::Number(255.0)));
        assert_eq!(
            result.channels[1],
            Some(ColorValue::Percentage(Percentage::new(50.0)))
        );
        assert_eq!(result.channels[2], Some(ColorValue::Number(0.0)));
        assert_eq!(result.alpha, Alpha(1.0));
    }
}
