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

/// Always in degrees
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hue(f32);

impl Hue {
    pub fn value(&self) -> f32 {
        self.0
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorValue {
    Number(f32),
    Percentage(Percentage),
}

pub enum Fraction {
    Unsigned,
    Signed,
}

impl ColorValue {
    pub fn value(&self, range: RangeInclusive<f32>, fraction: Fraction) -> f32 {
        match self {
            ColorValue::Number(n) => n.clamp(*range.start(), *range.end()),
            ColorValue::Percentage(p) => match fraction {
                Fraction::Unsigned => Self::lerp(p.as_fraction(), range),
                Fraction::Signed => Self::signed_lerp(p.as_fraction(), range),
            },
        }
    }

    fn lerp(fraction: f32, range: RangeInclusive<f32>) -> f32 {
        let t = fraction.clamp(0.0, 1.0);
        *range.start() + t * (*range.end() - *range.start())
    }

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

/// Always -1.0..=1.0
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alpha(f32);

impl Alpha {
    pub fn value(&self) -> f32 {
        self.0
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
        Self((value.as_fraction()).clamp(-1.0, 1.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FunctionColor {
    Srgba(SRGBAColor),
    Cielab(Cielab),
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

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Ok(srgba) = value.parse::<SRGBAColor>() {
            return Ok(Self::Srgba(srgba));
        }

        if let Ok(cielab) = value.parse::<Cielab>() {
            return Ok(Self::Cielab(cielab));
        }

        if let Ok(oklab) = value.parse::<Oklab>() {
            return Ok(Self::Oklab(oklab));
        }

        Err(format!("'{}', Invalid functional color format", value))
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
