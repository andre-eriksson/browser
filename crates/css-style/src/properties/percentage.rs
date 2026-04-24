use css_values::numeric::Percentage;

use crate::{AbsoluteContext, RelativeContext, RelativeType, properties::PixelRepr};

impl PixelRepr for Percentage {
    fn to_px(
        &self,
        rel_type: Option<RelativeType>,
        rel_ctx: Option<&RelativeContext>,
        abs_ctx: &AbsoluteContext,
    ) -> f64 {
        match rel_type {
            Some(val) => match val {
                RelativeType::FontSize => rel_ctx
                    .map_or(abs_ctx.root_font_size * self.as_fraction(), |ctx| ctx.font_size * self.as_fraction()),
                RelativeType::ParentHeight => rel_ctx.map_or(abs_ctx.viewport_height * self.as_fraction(), |ctx| {
                    ctx.parent.intrinsic_height * self.as_fraction()
                }),
                RelativeType::ParentWidth => rel_ctx.map_or(abs_ctx.viewport_width * self.as_fraction(), |ctx| {
                    ctx.parent.intrinsic_width * self.as_fraction()
                }),
                RelativeType::RootFontSize => abs_ctx.root_font_size * self.as_fraction(),
                RelativeType::ViewportHeight => abs_ctx.viewport_height * self.as_fraction(),
                RelativeType::ViewportWidth => abs_ctx.viewport_width * self.as_fraction(),
                RelativeType::BackgroundArea => {
                    let bg_area = rel_ctx.map_or_else(
                        || abs_ctx.viewport_width * abs_ctx.viewport_height,
                        |ctx| ctx.parent.intrinsic_width * ctx.parent.intrinsic_height,
                    );
                    bg_area.sqrt() * self.as_fraction()
                }
            },
            None => 0.0,
        }
    }
}
