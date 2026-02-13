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
        match value.first() {
            Some(ComponentValue::Token(token)) => match &token.kind {
                CssTokenKind::Ident(ident) if ident.eq_ignore_ascii_case("currentColor") => {
                    return Ok(Self::Current);
                }
                CssTokenKind::Ident(ident) if ident.eq_ignore_ascii_case("transparent") => {
                    return Ok(Self::Transparent);
                }
                _ => {}
            },
            _ => {}
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
