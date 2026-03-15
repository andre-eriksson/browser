//! Defines the Dimension and MaxDimension types, which represent CSS dimension values (width, height, max-width, max-height) and their parsing from CSS component values.

use css_values::dimension::{Dimension, MaxDimension};

use crate::properties::{AbsoluteContext, PixelRepr, RelativeContext, RelativeType};

impl PixelRepr for Dimension {
    fn to_px(
        &self,
        rel_type: Option<RelativeType>,
        rel_ctx: Option<&RelativeContext>,
        abs_ctx: &AbsoluteContext,
    ) -> f32 {
        match self {
            Dimension::Length(l) => l.to_px(rel_type, rel_ctx, abs_ctx),
            Dimension::MaxContent => 0.0,
            Dimension::MinContent => 0.0,
            Dimension::FitContent(_) => 0.0,
            Dimension::Stretch => 0.0,
            Dimension::Auto => 0.0,
            Dimension::Calc(calc) => calc.to_px(rel_type, rel_ctx, abs_ctx),
            Dimension::Percentage(p) => match rel_type {
                Some(RelativeType::FontSize) => rel_ctx
                    .map(|ctx| ctx.font_size * p.as_fraction())
                    .unwrap_or(abs_ctx.root_font_size * p.as_fraction()),
                Some(RelativeType::ParentHeight) => rel_ctx
                    .map(|ctx| ctx.parent.intrinsic_height * p.as_fraction())
                    .unwrap_or(abs_ctx.viewport_height * p.as_fraction()),
                Some(RelativeType::ParentWidth) => rel_ctx
                    .map(|ctx| ctx.parent.intrinsic_width * p.as_fraction())
                    .unwrap_or(abs_ctx.viewport_width * p.as_fraction()),
                Some(RelativeType::RootFontSize) => abs_ctx.root_font_size * p.as_fraction(),
                Some(RelativeType::ViewportHeight) => abs_ctx.viewport_height * p.as_fraction(),
                Some(RelativeType::ViewportWidth) => abs_ctx.viewport_width * p.as_fraction(),
                None => 0.0,
            },
        }
    }
}

impl PixelRepr for MaxDimension {
    fn to_px(
        &self,
        rel_type: Option<RelativeType>,
        rel_ctx: Option<&RelativeContext>,
        abs_ctx: &AbsoluteContext,
    ) -> f32 {
        match self {
            MaxDimension::Length(l) => l.to_px(rel_type, rel_ctx, abs_ctx),
            MaxDimension::MaxContent => 0.0,
            MaxDimension::MinContent => 0.0,
            MaxDimension::FitContent(_) => 0.0,
            MaxDimension::Stretch => 0.0,
            MaxDimension::None => f32::INFINITY,
            MaxDimension::Calc(calc) => calc.to_px(rel_type, rel_ctx, abs_ctx),
            MaxDimension::Percentage(p) => match rel_type {
                Some(RelativeType::FontSize) => rel_ctx
                    .map(|ctx| ctx.font_size * p.as_fraction())
                    .unwrap_or(abs_ctx.root_font_size * p.as_fraction()),
                Some(RelativeType::ParentHeight) => rel_ctx
                    .map(|ctx| ctx.parent.intrinsic_height * p.as_fraction())
                    .unwrap_or(abs_ctx.viewport_height * p.as_fraction()),
                Some(RelativeType::ParentWidth) => rel_ctx
                    .map(|ctx| ctx.parent.intrinsic_width * p.as_fraction())
                    .unwrap_or(abs_ctx.viewport_width * p.as_fraction()),
                Some(RelativeType::RootFontSize) => abs_ctx.root_font_size * p.as_fraction(),
                Some(RelativeType::ViewportHeight) => abs_ctx.viewport_height * p.as_fraction(),
                Some(RelativeType::ViewportWidth) => abs_ctx.viewport_width * p.as_fraction(),
                None => 0.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use css_values::numeric::Percentage;

    use crate::ComputedStyle;

    use super::*;

    #[test]
    fn test_dimension_to_px() {
        let rel_ctx = RelativeContext {
            parent: Arc::new(ComputedStyle {
                font_size: 16.0,
                intrinsic_width: 200.0,
                ..Default::default()
            }),
            font_size: 16.0,
        };
        let abs_ctx = AbsoluteContext {
            root_font_size: 16.0,
            viewport_width: 800.0,
            viewport_height: 600.0,
            ..Default::default()
        };

        let dim = Dimension::Percentage(Percentage::new(50.0));
        assert_eq!(dim.to_px(Some(RelativeType::ParentWidth), Some(&rel_ctx), &abs_ctx), 100.0);
    }
}
