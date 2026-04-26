use css_values::numeric::Percentage;

use crate::{AbsoluteContext, ComputedSize, RelativeContext, RelativeType, properties::PixelRepr};

impl PixelRepr for Percentage {
    fn to_px(
        self,
        rel_type: Option<RelativeType>,
        rel_ctx: Option<&RelativeContext>,
        abs_ctx: &AbsoluteContext,
    ) -> Result<f64, String> {
        Ok(match rel_type {
            Some(val) => match val {
                RelativeType::FontSize => rel_ctx
                    .map_or(abs_ctx.root_font_size * self.as_fraction(), |ctx| ctx.font_size * self.as_fraction()),
                RelativeType::ParentHeight => {
                    let Some(rel) = rel_ctx else {
                        return Err("Percentage with ParentHeight relative type requires a RelativeContext".to_string());
                    };

                    match rel.parent.height {
                        ComputedSize::Px(px) => px * self.as_fraction(),
                        _ => Err("Parent height is not a fixed pixel value, cannot resolve percentage".to_string())?,
                    }
                }
                RelativeType::ParentWidth => {
                    let Some(rel) = rel_ctx else {
                        return Err("Percentage with ParentWidth relative type requires a RelativeContext".to_string());
                    };

                    match rel.parent.width {
                        ComputedSize::Px(px) => px * self.as_fraction(),
                        _ => Err("Parent width is not a fixed pixel value, cannot resolve percentage".to_string())?,
                    }
                }
                RelativeType::RootFontSize => abs_ctx.root_font_size * self.as_fraction(),
                RelativeType::ViewportHeight => abs_ctx.viewport_height * self.as_fraction(),
                RelativeType::ViewportWidth => abs_ctx.viewport_width * self.as_fraction(),
                RelativeType::BackgroundArea => {
                    let Some(rel) = rel_ctx else {
                        return Err(
                            "Percentage with BackgroundArea relative type requires a RelativeContext".to_string()
                        );
                    };

                    let width = match rel.parent.width {
                        ComputedSize::Px(px) => px,
                        _ => Err("Parent width is not a fixed pixel value, cannot resolve background area percentage"
                            .to_string())?,
                    };

                    let height = match rel.parent.height {
                        ComputedSize::Px(px) => px,
                        _ => {
                            Err("Parent height is not a fixed pixel value, cannot resolve background area percentage"
                                .to_string())?
                        }
                    };

                    let bg_area = width * height;
                    bg_area.sqrt() * self.as_fraction()
                }
            },
            None => 0.0,
        })
    }
}
