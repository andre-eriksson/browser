//! Properties related to text layout and formatting, such as `writing-mode`, `text-align`, `white-space`, and `line-height`.

use css_values::text::LineHeight;

use crate::{AbsoluteContext, RelativeContext, RelativeType, properties::PixelRepr};

impl PixelRepr for LineHeight {
    fn to_px(&self, rel_type: Option<RelativeType>, rel_ctx: &RelativeContext, abs_ctx: &AbsoluteContext) -> f32 {
        match self {
            LineHeight::Normal => rel_ctx.parent.font_size * 1.2,
            LineHeight::Number(num) => rel_ctx.parent.font_size * num,
            LineHeight::Length(len) => len.to_px(rel_type, rel_ctx, abs_ctx),
            LineHeight::Percentage(pct) => pct.as_fraction() * rel_ctx.parent.font_size,
            LineHeight::Calc(calc) => calc.to_px(Some(RelativeType::FontSize), rel_ctx, abs_ctx),
        }
    }
}
