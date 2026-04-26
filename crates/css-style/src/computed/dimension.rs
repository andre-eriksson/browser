use css_values::{
    calc::CalcKind,
    dimension::{MaxSize, Size},
    numeric::Percentage,
    quantity::Length,
};

use crate::{AbsoluteContext, RelativeContext, RelativeType, properties::PixelRepr};

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub enum ComputedSize {
    #[default]
    Auto,
    Px(f64),
    Percentage(f64),
    MaxContent,
    MinContent,
    FitContent,
    Stretch,
}

impl ComputedSize {
    pub fn resolve(
        size: Size,
        relative_type: RelativeType,
        relative_ctx: &RelativeContext,
        absolute_ctx: &AbsoluteContext,
    ) -> Result<Self, String> {
        Ok(match size {
            Size::Auto => Self::Auto,
            Size::Length(length) => {
                let px = length.to_px(Some(relative_type), Some(relative_ctx), absolute_ctx)?;
                Self::Px(px)
            }
            Size::Percentage(p) => Self::Percentage(p.as_fraction()),
            Size::Calc(expr) => {
                if expr.evaluate().is_ok() {
                    if let Ok(kind) = expr.into_sum().kind() {
                        match kind {
                            CalcKind::Percentage(p) => Self::Percentage(p.as_fraction()),
                            _ => Self::default(),
                        }
                    } else {
                        Self::default()
                    }
                } else {
                    let sum = expr.to_px(Some(relative_type), Some(relative_ctx), absolute_ctx)?;
                    Self::Px(sum)
                }
            }
            Size::MaxContent => Self::MaxContent,
            Size::MinContent => Self::MinContent,
            Size::FitContent => Self::FitContent,
            Size::Stretch => Self::Stretch,
        })
    }
}

impl From<ComputedSize> for Size {
    fn from(value: ComputedSize) -> Self {
        match value {
            ComputedSize::Auto => Self::Auto,
            ComputedSize::Px(px) => Self::Length(Length::px(px)),
            ComputedSize::Percentage(_) => Self::Auto,
            ComputedSize::MaxContent => Self::MaxContent,
            ComputedSize::MinContent => Self::MinContent,
            ComputedSize::FitContent => Self::FitContent,
            ComputedSize::Stretch => Self::Stretch,
        }
    }
}

impl From<f64> for ComputedSize {
    fn from(value: f64) -> Self {
        Self::Px(value)
    }
}

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub enum ComputedMaxSize {
    #[default]
    None,
    Px(f64),
    Percentage(f64),
    MaxContent,
    MinContent,
    FitContent,
    Stretch,
}

impl ComputedMaxSize {
    pub fn resolve(
        max_size: MaxSize,
        relative_type: RelativeType,
        relative_ctx: &RelativeContext,
        absolute_ctx: &AbsoluteContext,
    ) -> Result<Self, String> {
        Ok(match max_size {
            MaxSize::None => Self::None,
            MaxSize::Length(length) => {
                let px = length.to_px(Some(relative_type), Some(relative_ctx), absolute_ctx)?;
                Self::Px(px)
            }
            MaxSize::Percentage(p) => Self::Percentage(p.as_fraction()),
            MaxSize::Calc(expr) => {
                if expr.evaluate().is_ok() {
                    if let Ok(kind) = expr.into_sum().kind() {
                        match kind {
                            CalcKind::Percentage(p) => Self::Percentage(p.as_fraction()),
                            _ => Self::default(),
                        }
                    } else {
                        Self::default()
                    }
                } else {
                    let sum = expr.to_px(Some(relative_type), Some(relative_ctx), absolute_ctx)?;
                    Self::Px(sum)
                }
            }
            MaxSize::MaxContent => Self::MaxContent,
            MaxSize::MinContent => Self::MinContent,
            MaxSize::FitContent => Self::FitContent,
            MaxSize::Stretch => Self::Stretch,
        })
    }
}

impl From<ComputedMaxSize> for MaxSize {
    fn from(value: ComputedMaxSize) -> Self {
        match value {
            ComputedMaxSize::None => Self::None,
            ComputedMaxSize::Px(px) => Self::Length(Length::px(px)),
            ComputedMaxSize::Percentage(pct) => Self::Percentage(Percentage::from_fraction(pct)),
            ComputedMaxSize::MaxContent => Self::MaxContent,
            ComputedMaxSize::MinContent => Self::MinContent,
            ComputedMaxSize::FitContent => Self::FitContent,
            ComputedMaxSize::Stretch => Self::Stretch,
        }
    }
}

impl From<f64> for ComputedMaxSize {
    fn from(value: f64) -> Self {
        Self::Px(value)
    }
}
