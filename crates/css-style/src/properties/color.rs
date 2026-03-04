//! Defines the `Color` enum, which represents a CSS color value. This includes system colors, named colors, hex colors, functional colors, and special values like `currentColor` and `transparent`.
//! The `Color` enum can be constructed from CSS component values and can be converted to a `Color4f` for rendering purposes.

use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};

use crate::{
    color::{
        Alpha, ColorValue,
        hex::HexColor,
        named::{NamedColor, SystemColor},
        srgba::SRGBAColor,
    },
    computed::color::Color4f,
    primitives::color::FunctionColor,
    properties::CSSParsable,
};

/// Represents a CSS color value, which can be a system color, named color, hex color, functional color, currentColor, or transparent.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/color_value>
#[derive(Debug, Clone, PartialEq)]
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

impl CSSParsable for Color {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();

        let color = if let Some(cv) = stream.next_cv() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("currentColor") {
                            Ok(Self::Current)
                        } else if ident.eq_ignore_ascii_case("transparent") {
                            Ok(Self::Transparent)
                        } else if let Ok(system_color) = ident.parse() {
                            Ok(Self::System(system_color))
                        } else if let Ok(named_color) = ident.parse() {
                            Ok(Self::Named(named_color))
                        } else {
                            Err(format!("Unrecognized color identifier: {}", ident))
                        }
                    }
                    CssTokenKind::Hash { .. } => {
                        let hex_color = HexColor::try_from(token)?;
                        Ok(Self::Hex(hex_color))
                    }
                    _ => Err("Expected an identifier or hash token for color".to_string()),
                },
                ComponentValue::Function(function) => {
                    let function_color = FunctionColor::try_from(function)?;
                    Ok(Self::Functional(function_color))
                }
                _ => Err("Unexpected component value type for color".to_string()),
            }
        } else {
            Err("Expected a component value for color".to_string())
        };

        stream.skip_whitespace();

        if stream.peek().is_some() {
            Err("Unexpected extra tokens after color value".to_string())
        } else {
            color
        }
    }
}
