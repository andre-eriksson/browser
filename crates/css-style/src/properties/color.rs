//! Defines the `Color` enum, which represents a CSS color value. This includes system colors, named colors, hex colors, functional colors, and special values like `currentColor` and `transparent`.
//! The `Color` enum can be constructed from CSS component values and can be converted to a `Color4f` for rendering purposes.

use css_cssom::{ComponentValue, CssTokenKind};

use crate::{
    color::{
        Alpha, ColorValue,
        hex::HexColor,
        named::{NamedColor, SystemColor},
        srgba::SRGBAColor,
    },
    computed::color::Color4f,
    primitives::color::FunctionColor,
};

/// Represents a CSS color value, which can be a system color, named color, hex color, functional color, currentColor, or transparent.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/color_value>
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

impl From<Color4f> for Color {
    fn from(color: Color4f) -> Self {
        Color::Functional(FunctionColor::Srgba(SRGBAColor::Rgb(
            ColorValue::Number(color.r * 255.0),
            ColorValue::Number(color.g * 255.0),
            ColorValue::Number(color.b * 255.0),
            Alpha::new(color.a),
        )))
    }
}

impl TryFrom<&[ComponentValue]> for Color {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            if let ComponentValue::Token(token) = cv
                && let CssTokenKind::Ident(ident) = &token.kind
            {
                if ident.eq_ignore_ascii_case("currentColor") {
                    return Ok(Self::Current);
                } else if ident.eq_ignore_ascii_case("transparent") {
                    return Ok(Self::Transparent);
                }
            }
        }

        if let Ok(hex_color) = value.try_into() {
            Ok(Self::Hex(hex_color))
        } else if let Ok(function_color) = value.try_into() {
            Ok(Self::Functional(function_color))
        } else if let Ok(system_color) = value.try_into() {
            Ok(Self::System(system_color))
        } else if let Ok(named_color) = value.try_into() {
            Ok(Self::Named(named_color))
        } else {
            Err("Invalid color value".to_string())
        }
    }
}
