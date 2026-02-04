use std::{fmt::Display, str::FromStr};

use crate::{
    color::{cielab::Cielab, oklab::Oklab, srgba::SRGBAColor},
    primitives::{angle::Angle, percentage::Percentage},
};

pub mod cielab;
pub mod hex;
pub mod named;
pub mod oklab;
pub mod srgba;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Hue {
    Angle(Angle),
    Number(f32),
}

impl From<f32> for Hue {
    fn from(value: f32) -> Self {
        Hue::Number(value)
    }
}

impl FromStr for Hue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Ok(angle) = s.parse::<Angle>() {
            Ok(Hue::Angle(angle))
        } else {
            let number = s.parse::<f32>().map_err(|e| e.to_string())?;
            Ok(Hue::Number(number))
        }
    }
}

pub trait AsFloat {
    fn as_float(&self) -> f32;
}

impl AsFloat for i8 {
    fn as_float(&self) -> f32 {
        *self as f32
    }
}

impl AsFloat for u8 {
    fn as_float(&self) -> f32 {
        *self as f32
    }
}

impl AsFloat for f32 {
    fn as_float(&self) -> f32 {
        *self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorValue<T: FromStr + AsFloat> {
    Number(T),
    Percentage(Percentage),
}

impl From<u8> for ColorValue<u8> {
    fn from(value: u8) -> Self {
        ColorValue::Number(value)
    }
}

impl<T: FromStr + AsFloat> ColorValue<T> {
    pub fn as_number(&self) -> f32 {
        match self {
            ColorValue::Number(num) => num.as_float(),
            ColorValue::Percentage(pct) => pct.value(),
        }
    }
}

impl<T: FromStr + AsFloat> FromStr for ColorValue<T>
where
    T::Err: Display,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.ends_with('%') {
            let percentage = s.parse::<Percentage>()?;
            Ok(ColorValue::Percentage(percentage))
        } else {
            let number = s.parse::<T>().map_err(|e| e.to_string())?;
            Ok(ColorValue::Number(number))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alpha {
    Number(f32),
    Percentage(Percentage),
}

impl Alpha {
    pub fn as_number(&self) -> f32 {
        match self {
            Alpha::Number(num) => *num,
            Alpha::Percentage(pct) => pct.value(),
        }
    }

    pub fn as_fraction(&self) -> f32 {
        match self {
            Alpha::Number(num) => *num,
            Alpha::Percentage(pct) => pct.as_fraction(),
        }
    }
}

impl FromStr for Alpha {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.ends_with('%') {
            let percentage = s.parse::<Percentage>()?;
            Ok(Alpha::Percentage(percentage))
        } else {
            let number = s.parse::<f32>().map_err(|e| e.to_string())?;
            Ok(Alpha::Number(number))
        }
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
}
