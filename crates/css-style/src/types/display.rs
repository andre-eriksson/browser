//! <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/display>

use crate::types::global::Global;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OutsideDisplay {
    Block,
    Inline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InsideDisplay {
    Flow,
    FlowRoot,
    Table,
    Flex,
    Grid,
    Ruby,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InternalDisplay {
    TableRowGroup,
    TableHeaderGroup,
    TableFooterGroup,
    TableRow,
    TableCell,
    TableColumnGroup,
    TableColumn,
    TableCaption,
    RubyBase,
    RubyText,
    RubyBaseContainer,
    RubyTextContainer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BoxDisplay {
    Contents,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Display {
    pub outside: Option<OutsideDisplay>,
    pub inside: Option<InsideDisplay>,
    pub internal: Option<InternalDisplay>,
    pub box_display: Option<BoxDisplay>,
    pub global: Option<Global>,
}
