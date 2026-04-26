use css_values::{
    calc::CalcKind,
    dimension::{MarginValue, OffsetValue},
    numeric::Percentage,
};

use crate::{AbsoluteContext, RelativeContext, RelativeType, properties::PixelRepr};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComputedOffset {
    Px(f64),
    Percentage(f64),
}

impl ComputedOffset {
    pub fn resolve(
        offset_value: OffsetValue,
        relative_type: Option<RelativeType>,
        relative_ctx: &RelativeContext,
        absolute_ctx: &AbsoluteContext,
    ) -> Result<Self, String> {
        Ok(match offset_value {
            OffsetValue::Length(len) => Self::Px(len.to_px(relative_type, Some(relative_ctx), absolute_ctx)?),
            OffsetValue::Percentage(pct) => Self::Percentage(pct.as_fraction()),
            OffsetValue::Calc(expr) => {
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
                    let sum = expr.to_px(relative_type, Some(relative_ctx), absolute_ctx)?;

                    Self::Px(sum)
                }
            }
        })
    }

    pub fn to_px(&self, containing_width: f64) -> f64 {
        match self {
            Self::Px(px) => *px,
            Self::Percentage(frac) => frac * containing_width,
        }
    }
}

impl Default for ComputedOffset {
    fn default() -> Self {
        Self::Px(0.0)
    }
}

impl From<ComputedOffset> for OffsetValue {
    fn from(value: ComputedOffset) -> Self {
        match value {
            ComputedOffset::Px(px) => Self::px(px),
            ComputedOffset::Percentage(frac) => Self::Percentage(Percentage::from_fraction(frac)),
        }
    }
}

impl From<f64> for ComputedOffset {
    fn from(value: f64) -> Self {
        Self::Px(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComputedMargin {
    Auto,
    Px(f64),
    Percentage(f64),
}

impl ComputedMargin {
    pub fn resolve(
        offset_value: MarginValue,
        relative_type: Option<RelativeType>,
        relative_ctx: &RelativeContext,
        absolute_ctx: &AbsoluteContext,
    ) -> Result<Self, String> {
        match offset_value {
            MarginValue::Auto => Ok(Self::Auto),
            MarginValue::Length(len) => Ok(Self::Px(len.to_px(relative_type, Some(relative_ctx), absolute_ctx)?)),
            MarginValue::Percentage(pct) => Ok(Self::Percentage(pct.as_fraction())),
            MarginValue::Calc(expr) => {
                let sum = expr.into_sum();

                Ok(match sum.kind() {
                    Ok(CalcKind::Length(len)) => {
                        Self::Px(len.to_px(relative_type, Some(relative_ctx), absolute_ctx)?)
                    }
                    Ok(CalcKind::Percentage(p)) => Self::Percentage(p.as_fraction()),
                    _ => Self::Auto,
                })
            }
        }
    }

    pub fn to_px(&self, containing_width: f64) -> f64 {
        match self {
            Self::Auto => 0.0,
            Self::Px(px) => *px,
            Self::Percentage(frac) => frac * containing_width,
        }
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }
}

impl Default for ComputedMargin {
    fn default() -> Self {
        Self::Px(0.0)
    }
}

impl From<ComputedMargin> for MarginValue {
    fn from(value: ComputedMargin) -> Self {
        match value {
            ComputedMargin::Auto => Self::Auto,
            ComputedMargin::Px(px) => Self::px(px),
            ComputedMargin::Percentage(frac) => Self::Percentage(Percentage::from_fraction(frac)),
        }
    }
}

impl From<f64> for ComputedMargin {
    fn from(value: f64) -> Self {
        Self::Px(value)
    }
}
