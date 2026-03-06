//! This module defines the `BorderWidth` and `BorderStyle` types, which represent the width and style of CSS borders, respectively.
//! These types can be constructed from CSS component values and can be converted to pixel values for rendering.

use css_values::border::BorderWidth;

use crate::properties::{AbsoluteContext, PixelRepr, RelativeContext};

impl PixelRepr for BorderWidth {
    fn to_px(
        &self,
        rel_type: Option<super::RelativeType>,
        rel_ctx: &RelativeContext,
        abs_ctx: &AbsoluteContext,
    ) -> f32 {
        match self {
            BorderWidth::Length(len) => len.to_px(rel_type, rel_ctx, abs_ctx),
            BorderWidth::Calc(calc) => calc.to_px(None, rel_ctx, abs_ctx),
            BorderWidth::Thin => 1.0,
            BorderWidth::Medium => 3.0,
            BorderWidth::Thick => 5.0,
        }
    }
}
