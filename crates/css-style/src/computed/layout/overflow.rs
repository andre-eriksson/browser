use css_values::OverflowBlock;

use crate::properties::OverflowProperty;

/// Computes the overflow properties for the given specified values.
///
/// # Rules
/// overflow-x: as specified, except with visible/clip computing to auto/hidden
/// respectively if one of overflow-x or overflow-y is neither visible nor clip
/// overflow-y: as specified, except with visible/clip computing to auto/hidden
/// respectively if one of overflow-x or overflow-y is neither visible nor clip
pub fn compute_overflow(
    specified_overflow_x: OverflowProperty,
    specified_overflow_y: OverflowProperty,
) -> (OverflowBlock, OverflowBlock) {
    let mut overflow_x = specified_overflow_x.compute(OverflowBlock::Visible);
    let mut overflow_y = specified_overflow_y.compute(OverflowBlock::Visible);

    let is_neither_visible_nor_clip = |val: OverflowBlock| val != OverflowBlock::Visible && val != OverflowBlock::Clip;

    if is_neither_visible_nor_clip(overflow_x) || is_neither_visible_nor_clip(overflow_y) {
        if overflow_x == OverflowBlock::Visible {
            overflow_x = OverflowBlock::Auto;
        } else if overflow_x == OverflowBlock::Clip {
            overflow_x = OverflowBlock::Hidden;
        }

        if overflow_y == OverflowBlock::Visible {
            overflow_y = OverflowBlock::Auto;
        } else if overflow_y == OverflowBlock::Clip {
            overflow_y = OverflowBlock::Hidden;
        }
    }

    (overflow_x, overflow_y)
}
