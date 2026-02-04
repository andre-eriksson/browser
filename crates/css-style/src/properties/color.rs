use std::str::FromStr;

use crate::{
    color::{
        hex::HexColor,
        named::{NamedColor, SystemColor},
    },
    primitives::color::FunctionColor,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    System(SystemColor),
    Named(NamedColor),
    Hex(HexColor),
    Functional(FunctionColor),
    Current,
    Transparent,
}

impl Default for Color {
    fn default() -> Self {
        Color::Named(NamedColor::Black)
    }
}

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.eq_ignore_ascii_case("currentColor") {
            Ok(Self::Current)
        } else if s.eq_ignore_ascii_case("transparent") {
            Ok(Self::Transparent)
        } else if s.starts_with('#')
            && let Ok(hex_color) = s.parse()
        {
            Ok(Self::Hex(hex_color))
        } else if let Ok(function_color) = s.parse() {
            Ok(Self::Functional(function_color))
        } else if let Ok(system_color) = s.parse() {
            Ok(Self::System(system_color))
        } else if let Ok(named_color) = s.parse() {
            Ok(Self::Named(named_color))
        } else {
            Err(format!("Invalid color value: {}", s))
        }
    }
}
