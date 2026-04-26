use css_values::{
    background::{Size, WidthHeightSize},
    calc::CalcKind,
    combination::LengthPercentage,
    numeric::Percentage,
    quantity::Length,
};

use crate::{
    AbsoluteContext, RelativeContext, RelativeType,
    properties::{PixelRepr, background::BackgroundSize},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComputedLengthPercentage {
    Px(f64),
    Percentage(f64),
}

impl ComputedLengthPercentage {
    pub fn resolve(
        len_pct: LengthPercentage,
        relative_type: Option<RelativeType>,
        relative_ctx: &RelativeContext,
        absolute_ctx: &AbsoluteContext,
    ) -> Result<Self, String> {
        match len_pct {
            LengthPercentage::Length(len) => {
                Ok(Self::Px(len.to_px(relative_type, Some(relative_ctx), absolute_ctx)?))
            }
            LengthPercentage::Percentage(pct) => Ok(Self::Percentage(pct.as_fraction())),
            LengthPercentage::Calc(expr) => {
                let sum = expr.into_sum();

                match sum.kind() {
                    Ok(CalcKind::Length(len)) => {
                        let px = len.to_px(relative_type, Some(relative_ctx), absolute_ctx)?;
                        Ok(Self::Px(px))
                    }
                    Ok(CalcKind::Percentage(p)) => Ok(Self::Percentage(p.as_fraction())),
                    _ => Err("Unsupported calc expression for LengthPercentage".to_string()),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComputedWidthHeightSize {
    Auto,
    Length(ComputedLengthPercentage),
}

impl ComputedWidthHeightSize {
    pub fn resolve(
        width_height_size: WidthHeightSize,
        relative_type: Option<RelativeType>,
        relative_ctx: &RelativeContext,
        absolute_ctx: &AbsoluteContext,
    ) -> Self {
        match width_height_size {
            WidthHeightSize::Auto => Self::Auto,
            WidthHeightSize::Length(len_pct) => {
                match ComputedLengthPercentage::resolve(len_pct, relative_type, relative_ctx, absolute_ctx) {
                    Ok(resolved) => Self::Length(resolved),
                    Err(_) => Self::Auto,
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComputedSize {
    Cover,
    Contain,
    WidthHeight(ComputedWidthHeightSize, Option<ComputedWidthHeightSize>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedBackgroundSize(pub Vec<ComputedSize>);

impl Default for ComputedBackgroundSize {
    fn default() -> Self {
        Self(vec![ComputedSize::WidthHeight(
            ComputedWidthHeightSize::Auto,
            Some(ComputedWidthHeightSize::Auto),
        )])
    }
}

impl ComputedBackgroundSize {
    pub fn resolve(
        background_size: BackgroundSize,
        relative_type: Option<RelativeType>,
        relative_ctx: &RelativeContext,
        absolute_ctx: &AbsoluteContext,
    ) -> Self {
        let sizes = background_size
            .0
            .into_iter()
            .map(|size| match size {
                Size::Cover => ComputedSize::Cover,
                Size::Contain => ComputedSize::Contain,
                Size::WidthHeight(width, height) => {
                    let width = ComputedWidthHeightSize::resolve(width, relative_type, relative_ctx, absolute_ctx);
                    let height =
                        height.map(|h| ComputedWidthHeightSize::resolve(h, relative_type, relative_ctx, absolute_ctx));
                    ComputedSize::WidthHeight(width, height)
                }
            })
            .collect();
        Self(sizes)
    }
}

impl From<ComputedBackgroundSize> for BackgroundSize {
    fn from(value: ComputedBackgroundSize) -> Self {
        fn resolve_width_height_size(computed: ComputedWidthHeightSize) -> WidthHeightSize {
            match computed {
                ComputedWidthHeightSize::Auto => WidthHeightSize::Auto,
                ComputedWidthHeightSize::Length(len_pct) => WidthHeightSize::Length(match len_pct {
                    ComputedLengthPercentage::Px(px) => LengthPercentage::Length(Length::px(px)),
                    ComputedLengthPercentage::Percentage(frac) => {
                        LengthPercentage::Percentage(Percentage::from_fraction(frac))
                    }
                }),
            }
        }

        let sizes = value
            .0
            .into_iter()
            .map(|size| match size {
                ComputedSize::Cover => Size::Cover,
                ComputedSize::Contain => Size::Contain,
                ComputedSize::WidthHeight(width, height) => {
                    let width = resolve_width_height_size(width);
                    let height = height.map(resolve_width_height_size);

                    Size::WidthHeight(width, height)
                }
            })
            .collect();
        Self(sizes)
    }
}
