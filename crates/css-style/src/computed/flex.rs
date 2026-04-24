use css_values::flex::FlexBasis;

use crate::ComputedSize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComputedFlexBasis {
    Content,
    Size(ComputedSize),
}

impl Default for ComputedFlexBasis {
    fn default() -> Self {
        Self::Size(ComputedSize::Auto)
    }
}

impl From<FlexBasis> for ComputedFlexBasis {
    fn from(value: FlexBasis) -> Self {
        match value {
            FlexBasis::Content => Self::Content,
            FlexBasis::Size(size) => Self::Size(size.into()),
        }
    }
}

impl From<ComputedFlexBasis> for FlexBasis {
    fn from(value: ComputedFlexBasis) -> Self {
        match value {
            ComputedFlexBasis::Content => Self::Content,
            ComputedFlexBasis::Size(size) => Self::Size(size.into()),
        }
    }
}
