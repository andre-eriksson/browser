use std::str::FromStr;

use crate::primitives::color::{FunctionColor, NamedColor, SRGBAColor, SystemColor};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    System(SystemColor),
    Named(NamedColor),
    Hex(SRGBAColor),
    Functional(FunctionColor),
    Current,
    Transparent,
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
            && let Ok(hex_color) = s.parse::<SRGBAColor>()
        {
            Ok(Self::Hex(hex_color))
        } else if let Ok(function_color) = s.parse::<FunctionColor>() {
            Ok(Self::Functional(function_color))
        } else if let Ok(system_color) = s.parse::<SystemColor>() {
            Ok(Self::System(system_color))
        } else if let Ok(named_color) = s.parse::<NamedColor>() {
            Ok(Self::Named(named_color))
        } else {
            Err(format!("Invalid color value: {}", s))
        }
    }
}
