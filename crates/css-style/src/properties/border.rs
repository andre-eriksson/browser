//! This module defines the `BorderWidth` and `BorderStyle` types, which represent the width and style of CSS borders, respectively.
//! These types can be constructed from CSS component values and can be converted to pixel values for rendering.

use css_values::{border::BorderWidth, calc::CalcKind};

use crate::{
    RelativeType,
    properties::{AbsoluteContext, PixelRepr, RelativeContext},
};

impl PixelRepr for BorderWidth {
    fn to_px(
        self,
        rel_type: Option<RelativeType>,
        rel_ctx: Option<&RelativeContext>,
        abs_ctx: &AbsoluteContext,
    ) -> Result<f64, String> {
        Ok(match self {
            Self::Length(len) => len.to_px(rel_type, rel_ctx, abs_ctx)?,
            Self::Calc(expr) => {
                let kind = expr.into_sum().kind();

                match kind {
                    Ok(CalcKind::Length(len)) => len.to_px(rel_type, rel_ctx, abs_ctx)?,
                    _ => 0.0,
                }
            }
            Self::Thin => 1.0,
            Self::Medium => 3.0,
            Self::Thick => 5.0,
        })
    }
}
