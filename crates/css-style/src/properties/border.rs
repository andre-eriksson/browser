use std::str::FromStr;

use strum::EnumString;

use crate::{primitives::length::Length, properties::color::Color};

#[derive(Debug, Clone, Copy)]
pub enum BorderWidthValue {
    Length(Length),
    Thin,
    Medium,
    Thick,
}

impl FromStr for BorderWidthValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(length) = s.parse() {
            Ok(BorderWidthValue::Length(length))
        } else if s.eq_ignore_ascii_case("thin") {
            Ok(BorderWidthValue::Thin)
        } else if s.eq_ignore_ascii_case("medium") {
            Ok(BorderWidthValue::Medium)
        } else if s.eq_ignore_ascii_case("thick") {
            Ok(BorderWidthValue::Thick)
        } else {
            Err(format!("Invalid border width value: {}", s))
        }
    }
}

#[derive(Debug, Clone)]
pub struct BorderWidth {
    top: BorderWidthValue,
    right: BorderWidthValue,
    bottom: BorderWidthValue,
    left: BorderWidthValue,
}

impl BorderWidth {
    pub fn zero() -> Self {
        Self {
            top: BorderWidthValue::Length(Length::zero()),
            right: BorderWidthValue::Length(Length::zero()),
            bottom: BorderWidthValue::Length(Length::zero()),
            left: BorderWidthValue::Length(Length::zero()),
        }
    }

    pub fn new(
        top: BorderWidthValue,
        right: BorderWidthValue,
        bottom: BorderWidthValue,
        left: BorderWidthValue,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn all(value: BorderWidthValue) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn top(&self) -> BorderWidthValue {
        self.top
    }

    pub fn right(&self) -> BorderWidthValue {
        self.right
    }

    pub fn bottom(&self) -> BorderWidthValue {
        self.bottom
    }

    pub fn left(&self) -> BorderWidthValue {
        self.left
    }
}

impl FromStr for BorderWidth {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split_whitespace().collect::<Vec<&str>>();

        match parts.len() {
            1 => {
                let width = parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid border width: {}", parts[0]))?;

                Ok(BorderWidth {
                    top: width,
                    right: width,
                    bottom: width,
                    left: width,
                })
            }
            2 => {
                let vertical = parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid border width: {}", parts[0]))?;
                let horizontal = parts[1]
                    .parse()
                    .map_err(|_| format!("Invalid border width: {}", parts[1]))?;
                Ok(BorderWidth {
                    top: vertical,
                    right: horizontal,
                    bottom: vertical,
                    left: horizontal,
                })
            }
            3 => {
                let top = parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid border width: {}", parts[0]))?;
                let horizontal = parts[1]
                    .parse()
                    .map_err(|_| format!("Invalid border width: {}", parts[1]))?;
                let bottom = parts[2]
                    .parse()
                    .map_err(|_| format!("Invalid border width: {}", parts[2]))?;
                Ok(BorderWidth {
                    top,
                    right: horizontal,
                    bottom,
                    left: horizontal,
                })
            }
            4 => {
                let top = parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid border width: {}", parts[0]))?;
                let right = parts[1]
                    .parse()
                    .map_err(|_| format!("Invalid border width: {}", parts[1]))?;
                let bottom = parts[2]
                    .parse()
                    .map_err(|_| format!("Invalid border width: {}", parts[2]))?;
                let left = parts[3]
                    .parse()
                    .map_err(|_| format!("Invalid border width: {}", parts[3]))?;
                Ok(BorderWidth {
                    top,
                    right,
                    bottom,
                    left,
                })
            }
            _ => Err("Invalid number of width values for BorderWidth".to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum BorderStyleValue {
    None,
    Hidden,
    Dotted,
    Dashed,
    Solid,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorderStyle {
    top: BorderStyleValue,
    right: BorderStyleValue,
    bottom: BorderStyleValue,
    left: BorderStyleValue,
}

impl BorderStyle {
    pub fn none() -> Self {
        Self {
            top: BorderStyleValue::None,
            right: BorderStyleValue::None,
            bottom: BorderStyleValue::None,
            left: BorderStyleValue::None,
        }
    }

    pub fn all(value: BorderStyleValue) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn top(&self) -> BorderStyleValue {
        self.top
    }

    pub fn right(&self) -> BorderStyleValue {
        self.right
    }

    pub fn bottom(&self) -> BorderStyleValue {
        self.bottom
    }

    pub fn left(&self) -> BorderStyleValue {
        self.left
    }
}

impl FromStr for BorderStyle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split_whitespace().collect::<Vec<&str>>();

        match parts.len() {
            1 => {
                let style = parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid border style: {}", parts[0]))?;

                Ok(BorderStyle {
                    top: style,
                    right: style,
                    bottom: style,
                    left: style,
                })
            }
            2 => {
                let top_bottom = parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid border style: {}", parts[0]))?;
                let left_right = parts[1]
                    .parse()
                    .map_err(|_| format!("Invalid border style: {}", parts[1]))?;
                Ok(BorderStyle {
                    top: top_bottom,
                    right: left_right,
                    bottom: top_bottom,
                    left: left_right,
                })
            }
            3 => {
                let top = parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid border style: {}", parts[0]))?;
                let left_right = parts[1]
                    .parse()
                    .map_err(|_| format!("Invalid border style: {}", parts[1]))?;
                let bottom = parts[2]
                    .parse()
                    .map_err(|_| format!("Invalid border style: {}", parts[2]))?;
                Ok(BorderStyle {
                    top,
                    right: left_right,
                    bottom,
                    left: left_right,
                })
            }
            4 => {
                let top = parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid border style: {}", parts[0]))?;
                let right = parts[1]
                    .parse()
                    .map_err(|_| format!("Invalid border style: {}", parts[1]))?;
                let bottom = parts[2]
                    .parse()
                    .map_err(|_| format!("Invalid border style: {}", parts[2]))?;
                let left = parts[3]
                    .parse()
                    .map_err(|_| format!("Invalid border style: {}", parts[3]))?;
                Ok(BorderStyle {
                    top,
                    right,
                    bottom,
                    left,
                })
            }
            _ => Err("Invalid number of style values for BorderStyle".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BorderColor {
    top: Color,
    right: Color,
    bottom: Color,
    left: Color,
}

impl BorderColor {
    pub fn all(value: Color) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn top(&self) -> &Color {
        &self.top
    }

    pub fn right(&self) -> &Color {
        &self.right
    }

    pub fn bottom(&self) -> &Color {
        &self.bottom
    }

    pub fn left(&self) -> &Color {
        &self.left
    }
}

impl FromStr for BorderColor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split_whitespace().collect::<Vec<&str>>();

        match parts.len() {
            1 => {
                let color = parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid color: {}", parts[0]))?;

                Ok(BorderColor {
                    top: color,
                    right: color,
                    bottom: color,
                    left: color,
                })
            }
            2 => {
                let top_bottom = parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid color: {}", parts[0]))?;
                let left_right = parts[1]
                    .parse()
                    .map_err(|_| format!("Invalid color: {}", parts[1]))?;
                Ok(BorderColor {
                    top: top_bottom,
                    right: left_right,
                    bottom: top_bottom,
                    left: left_right,
                })
            }
            3 => {
                let top = parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid color: {}", parts[0]))?;
                let left_right = parts[1]
                    .parse()
                    .map_err(|_| format!("Invalid color: {}", parts[1]))?;
                let bottom = parts[2]
                    .parse()
                    .map_err(|_| format!("Invalid color: {}", parts[2]))?;
                Ok(BorderColor {
                    top,
                    right: left_right,
                    bottom,
                    left: left_right,
                })
            }
            4 => {
                let top = parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid color: {}", parts[0]))?;
                let right = parts[1]
                    .parse()
                    .map_err(|_| format!("Invalid color: {}", parts[1]))?;
                let bottom = parts[2]
                    .parse()
                    .map_err(|_| format!("Invalid color: {}", parts[2]))?;
                let left = parts[3]
                    .parse()
                    .map_err(|_| format!("Invalid color: {}", parts[3]))?;
                Ok(BorderColor {
                    top,
                    right,
                    bottom,
                    left,
                })
            }
            _ => Err("Invalid number of color values for BorderColor".to_string()),
        }
    }
}
