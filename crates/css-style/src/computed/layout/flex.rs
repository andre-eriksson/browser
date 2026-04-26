use css_values::FlexBasis;

use crate::{AbsoluteContext, ComputedSize, RelativeContext, RelativeType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComputedFlexBasis {
    Content,
    Size(ComputedSize),
}

impl ComputedFlexBasis {
    pub fn resolve(
        flex_basis: FlexBasis,
        relative_type: RelativeType,
        relative_ctx: &RelativeContext,
        absolute_ctx: &AbsoluteContext,
    ) -> Result<Self, String> {
        Ok(match flex_basis {
            FlexBasis::Content => Self::Content,
            FlexBasis::Size(size) => {
                Self::Size(ComputedSize::resolve(size, relative_type, relative_ctx, absolute_ctx)?)
            }
        })
    }
}

impl Default for ComputedFlexBasis {
    fn default() -> Self {
        Self::Size(ComputedSize::Auto)
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
