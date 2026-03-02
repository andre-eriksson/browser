use crate::{
    AbsoluteContext, RelativeContext,
    length::Length,
    percentage::{LengthPercentage, Percentage},
    properties::background::{BackgroundSize, Size, WidthHeightSize},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComputedLengthPercentage {
    Length(f32),
    Percentage(f32),
}

impl ComputedLengthPercentage {
    pub fn resolve(len_pct: LengthPercentage, relative_ctx: &RelativeContext, absolute_ctx: &AbsoluteContext) -> Self {
        match len_pct {
            LengthPercentage::Length(len) => Self::Length(len.to_px(relative_ctx, absolute_ctx)),
            LengthPercentage::Percentage(pct) => Self::Percentage(pct.as_fraction()),
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
        relative_ctx: &RelativeContext,
        absolute_ctx: &AbsoluteContext,
    ) -> Self {
        match width_height_size {
            WidthHeightSize::Auto => Self::Auto,
            WidthHeightSize::Length(len_pct) => {
                Self::Length(ComputedLengthPercentage::resolve(len_pct, relative_ctx, absolute_ctx))
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
pub struct ComputedBackgroundSize {
    pub sizes: Vec<ComputedSize>,
}

impl Default for ComputedBackgroundSize {
    fn default() -> Self {
        Self {
            sizes: vec![ComputedSize::WidthHeight(
                ComputedWidthHeightSize::Auto,
                Some(ComputedWidthHeightSize::Auto),
            )],
        }
    }
}

impl ComputedBackgroundSize {
    pub fn resolve(
        background_size: BackgroundSize,
        relative_ctx: &RelativeContext,
        absolute_ctx: &AbsoluteContext,
    ) -> Self {
        let sizes = background_size
            .sizes
            .into_iter()
            .map(|size| match size {
                Size::Cover => ComputedSize::Cover,
                Size::Contain => ComputedSize::Contain,
                Size::WidthHeight(width, height) => {
                    let width = ComputedWidthHeightSize::resolve(width, relative_ctx, absolute_ctx);
                    let height = height.map(|h| ComputedWidthHeightSize::resolve(h, relative_ctx, absolute_ctx));
                    ComputedSize::WidthHeight(width, height)
                }
            })
            .collect();
        Self { sizes }
    }
}

impl From<ComputedBackgroundSize> for BackgroundSize {
    fn from(value: ComputedBackgroundSize) -> Self {
        let sizes = value
            .sizes
            .into_iter()
            .map(|size| match size {
                ComputedSize::Cover => Size::Cover,
                ComputedSize::Contain => Size::Contain,
                ComputedSize::WidthHeight(width, height) => {
                    let width = match width {
                        ComputedWidthHeightSize::Auto => WidthHeightSize::Auto,
                        ComputedWidthHeightSize::Length(len_pct) => WidthHeightSize::Length(match len_pct {
                            ComputedLengthPercentage::Length(px) => LengthPercentage::Length(Length::px(px)),
                            ComputedLengthPercentage::Percentage(frac) => {
                                LengthPercentage::Percentage(Percentage::from_fraction(frac))
                            }
                        }),
                    };
                    let height = height.map(|h| match h {
                        ComputedWidthHeightSize::Auto => WidthHeightSize::Auto,
                        ComputedWidthHeightSize::Length(len_pct) => WidthHeightSize::Length(match len_pct {
                            ComputedLengthPercentage::Length(len) => LengthPercentage::Length(Length::px(len)),
                            ComputedLengthPercentage::Percentage(frac) => {
                                LengthPercentage::Percentage(Percentage::from_fraction(frac))
                            }
                        }),
                    });
                    Size::WidthHeight(width, height)
                }
            })
            .collect();
        Self { sizes }
    }
}
