//! Properties related to text layout and formatting, such as `writing-mode`, `text-align`, `white-space`, and `line-height`.

use css_values::{calc::CalcKind, text::LineHeight};

use crate::{AbsoluteContext, RelativeType, StyleContext, properties::PixelRepr};

impl PixelRepr for LineHeight {
    fn to_px(
        self,
        rel_type: Option<RelativeType>,
        style_ctx: Option<&StyleContext>,
        abs_ctx: &AbsoluteContext,
    ) -> Result<f64, String> {
        Ok(match self {
            Self::Normal => style_ctx.map_or(abs_ctx.root_font_size * abs_ctx.root_line_height_multiplier, |ctx| {
                ctx.font_size * abs_ctx.root_line_height_multiplier
            }),
            Self::Number(num) => style_ctx.map_or(abs_ctx.root_font_size * num, |ctx| ctx.font_size * num),
            Self::Length(len) => len.to_px(rel_type, style_ctx, abs_ctx)?,
            Self::Percentage(pct) => {
                style_ctx.map_or(abs_ctx.root_font_size * pct.as_fraction(), |ctx| ctx.font_size * pct.as_fraction())
            }
            Self::Calc(expr) => {
                let kind = expr.into_sum().kind();

                match kind {
                    Ok(CalcKind::Length(len)) => len.to_px(rel_type, style_ctx, abs_ctx)?,
                    Ok(CalcKind::Percentage(pct)) => style_ctx
                        .map_or(abs_ctx.root_font_size * pct.as_fraction(), |ctx| ctx.font_size * pct.as_fraction()),
                    Ok(CalcKind::Number(num)) => {
                        style_ctx.map_or(abs_ctx.root_font_size * num, |ctx| ctx.font_size * num)
                    }
                    _ => LineHeight::Normal.to_px(rel_type, style_ctx, abs_ctx)?,
                }
            }
        })
    }
}
