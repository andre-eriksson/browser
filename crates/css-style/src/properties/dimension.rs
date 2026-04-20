//! Defines the Dimension and `MaxDimension` types, which represent CSS dimension values (width, height, max-width, max-height) and their parsing from CSS component values.

use css_values::dimension::{Dimension, MaxDimension};

use crate::properties::{AbsoluteContext, PixelRepr, RelativeContext, RelativeType};

impl PixelRepr for Dimension {
    fn to_px(
        &self,
        rel_type: Option<RelativeType>,
        rel_ctx: Option<&RelativeContext>,
        abs_ctx: &AbsoluteContext,
    ) -> f64 {
        match self {
            Self::Length(l) => l.to_px(rel_type, rel_ctx, abs_ctx),
            Self::MaxContent => 0.0,
            Self::MinContent => 0.0,
            Self::FitContent(_) => 0.0,
            Self::Stretch => 0.0,
            Self::Auto => 0.0,
            Self::Calc(calc) => calc.to_px(rel_type, rel_ctx, abs_ctx),
            Self::Percentage(p) => match rel_type {
                Some(RelativeType::FontSize) => {
                    rel_ctx.map_or(abs_ctx.root_font_size * p.as_fraction(), |ctx| ctx.font_size * p.as_fraction())
                }
                Some(RelativeType::ParentHeight) => rel_ctx.map_or(abs_ctx.viewport_height * p.as_fraction(), |ctx| {
                    ctx.parent.intrinsic_height * p.as_fraction()
                }),
                Some(RelativeType::ParentWidth) => rel_ctx.map_or(abs_ctx.viewport_width * p.as_fraction(), |ctx| {
                    ctx.parent.intrinsic_width * p.as_fraction()
                }),
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
    ) -> f64 {
        match self {
            Self::Length(l) => l.to_px(rel_type, rel_ctx, abs_ctx),
            Self::MaxContent => 0.0,
            Self::MinContent => 0.0,
            Self::FitContent(_) => 0.0,
            Self::Stretch => 0.0,
            Self::None => f64::INFINITY,
            Self::Calc(calc) => calc.to_px(rel_type, rel_ctx, abs_ctx),
            Self::Percentage(p) => match rel_type {
                Some(RelativeType::FontSize) => {
                    rel_ctx.map_or(abs_ctx.root_font_size * p.as_fraction(), |ctx| ctx.font_size * p.as_fraction())
                }
                Some(RelativeType::ParentHeight) => rel_ctx.map_or(abs_ctx.viewport_height * p.as_fraction(), |ctx| {
                    ctx.parent.intrinsic_height * p.as_fraction()
                }),
                Some(RelativeType::ParentWidth) => rel_ctx.map_or(abs_ctx.viewport_width * p.as_fraction(), |ctx| {
                    ctx.parent.intrinsic_width * p.as_fraction()
                }),
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
    use std::{net::Ipv4Addr, sync::Arc};

    use css_values::numeric::Percentage;
    use url::Url;

    use crate::ComputedStyle;

    use super::*;

    #[test]
    fn test_dimension_to_px() {
        let url = Box::leak(Box::new(Url::parse(&format!("http://{}", Ipv4Addr::LOCALHOST)).unwrap()));
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
            ..AbsoluteContext::default_url(url)
        };

        let dim = Dimension::Percentage(Percentage::new(50.0));
        assert_eq!(dim.to_px(Some(RelativeType::ParentWidth), Some(&rel_ctx), &abs_ctx), 100.0);
    }
}
