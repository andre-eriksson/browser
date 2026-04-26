use css_values::{Gap, calc::CalcKind, numeric::Percentage, quantity::Length};

use crate::{AbsoluteContext, RelativeContext, RelativeType, properties::PixelRepr};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ComputedGap {
    #[default]
    Normal,
    Length(f64),
    Percentage(f64),
}

impl ComputedGap {
    pub fn resolve(
        gap: Gap,
        relative_type: RelativeType,
        relative_ctx: &RelativeContext,
        absolute_ctx: &AbsoluteContext,
    ) -> Result<Self, String> {
        Ok(match gap {
            Gap::Normal => Self::Normal,
            Gap::Length(length) => Self::Length(length.to_px(Some(relative_type), Some(relative_ctx), absolute_ctx)?),
            Gap::Percentage(p) => Self::Percentage(p.as_fraction()),
            Gap::Calc(expr) => {
                if expr.evaluate().is_ok() {
                    if let Ok(kind) = expr.into_sum().kind() {
                        match kind {
                            CalcKind::Percentage(p) => Self::Percentage(p.as_fraction()),
                            _ => Self::Normal,
                        }
                    } else {
                        Self::Normal
                    }
                } else {
                    let sum = expr.to_px(Some(relative_type), Some(relative_ctx), absolute_ctx)?;
                    Self::Length(sum)
                }
            }
        })
    }
}

impl From<ComputedGap> for Gap {
    fn from(value: ComputedGap) -> Self {
        match value {
            ComputedGap::Normal => Self::Normal,
            ComputedGap::Length(px) => Self::Length(Length::px(px)),
            ComputedGap::Percentage(frac) => Self::Percentage(Percentage::from_fraction(frac)),
        }
    }
}
