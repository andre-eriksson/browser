//! Defines the `Length` struct and related types for representing CSS length values.

use css_values::quantity::{Length, LengthUnit};

use crate::{
    RelativeType,
    properties::{AbsoluteContext, PixelRepr, RelativeContext},
};

impl PixelRepr for Length {
    fn to_px(
        &self,
        rel_type: Option<RelativeType>,
        rel_ctx: Option<&RelativeContext>,
        abs_ctx: &AbsoluteContext,
    ) -> f64 {
        match self.unit() {
            LengthUnit::Px => self.value(),
            LengthUnit::Cm => self.value() * 96.0 / 2.54,
            LengthUnit::Mm => self.value() * 96.0 / 25.4,
            LengthUnit::Q => self.value() * 96.0 / 101.6,
            LengthUnit::In => self.value() * 96.0,
            LengthUnit::Pc => self.value() * 16.0,
            LengthUnit::Pt => self.value() * 96.0 / 72.0,
            LengthUnit::Vw => abs_ctx.viewport_width * self.value() / 100.0,
            LengthUnit::Vh => abs_ctx.viewport_height * self.value() / 100.0,

            LengthUnit::Ch | LengthUnit::Cap => rel_ctx.map_or_else(
                || abs_ctx.root_font_size * 0.5 * self.value(),
                |ctx| ctx.parent.font_size * 0.5 * self.value(),
            ),
            LengthUnit::Rem => abs_ctx.root_font_size * self.value(),
            LengthUnit::Em => match rel_type {
                Some(RelativeType::FontSize) => rel_ctx
                    .map_or_else(|| abs_ctx.root_font_size * self.value(), |ctx| ctx.parent.font_size * self.value()),
                _ => rel_ctx.map_or_else(|| abs_ctx.root_font_size * self.value(), |ctx| ctx.font_size * self.value()),
            },
            _ => self.value(), // TODO: Handle other units properly
        }
    }
}
