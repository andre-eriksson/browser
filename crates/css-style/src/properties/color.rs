//! Defines the `Color` enum, which represents a CSS color value. This includes system colors, named colors, hex colors, functional colors, and special values like `currentColor` and `transparent`.
//! The `Color` enum can be constructed from CSS component values and can be converted to a `Color4f` for rendering purposes.

use css_values::color::{Alpha, Color, ColorValue, base::ColorBase, function::ColorFunction};

use crate::Color4f;

impl From<Color4f> for Color {
    fn from(color: Color4f) -> Self {
        Self::Base(ColorBase::Function(ColorFunction::Rgb(
            ColorValue::Number(color.r * 255.0),
            ColorValue::Number(color.g * 255.0),
            ColorValue::Number(color.b * 255.0),
            Alpha::new(color.a),
        )))
    }
}
