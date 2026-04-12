//! Properties related to text layout and formatting, such as `writing-mode`, `text-align`, `white-space`, and `line-height`.

use css_values::text::LineHeight;

use crate::{AbsoluteContext, RelativeContext, RelativeType, properties::PixelRepr};

impl PixelRepr for LineHeight {
    fn to_px(
        &self,
        rel_type: Option<RelativeType>,
        rel_ctx: Option<&RelativeContext>,
        abs_ctx: &AbsoluteContext,
    ) -> f32 {
        match self {
            Self::Normal => rel_ctx
                .map(|ctx| ctx.font_size * abs_ctx.root_line_height_multiplier)
                .unwrap_or(abs_ctx.root_font_size * abs_ctx.root_line_height_multiplier),
            Self::Number(num) => rel_ctx
                .map(|ctx| ctx.font_size * num)
                .unwrap_or(abs_ctx.root_font_size * num),
            Self::Length(len) => len.to_px(rel_type, rel_ctx, abs_ctx),
            Self::Percentage(pct) => rel_ctx
                .map(|ctx| ctx.font_size * pct.as_fraction())
                .unwrap_or(abs_ctx.root_font_size * pct.as_fraction()),
            Self::Calc(calc) => calc.to_px(Some(RelativeType::FontSize), rel_ctx, abs_ctx),
        }
    }
}
