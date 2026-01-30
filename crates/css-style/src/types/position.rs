//! <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/position>
use crate::types::global::Global;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
    Global(Global),
}
