use std::ops::Deref;

use css_style::ComputedStyle;
use css_values::display::OutsideDisplay;
use html_dom::NodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LayoutNodeId(usize);

impl LayoutNodeId {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

impl From<usize> for LayoutNodeId {
    fn from(index: usize) -> Self {
        Self::new(index)
    }
}

impl From<LayoutNodeId> for usize {
    fn from(id: LayoutNodeId) -> Self {
        id.index()
    }
}

#[derive(Debug, Clone)]
pub enum CopiedStyle<'a> {
    Defined(&'a ComputedStyle),
    Anonymous(Box<ComputedStyle>),
}

impl Deref for CopiedStyle<'_> {
    type Target = ComputedStyle;

    fn deref(&self) -> &Self::Target {
        match self {
            CopiedStyle::Defined(style) => style,
            CopiedStyle::Anonymous(style) => style,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoxNode<'a> {
    pub layout_id: LayoutNodeId,
    pub node_id: Option<NodeId>,
    pub style: CopiedStyle<'a>,
    pub children: Vec<LayoutNodeId>,
}

impl<'a> BoxNode<'a> {
    pub fn new(node_id: &'a NodeId, style: &'a ComputedStyle, _children: Vec<BoxNode<'a>>) -> Self {
        Self::new_with_layout_id(LayoutNodeId::new(0), node_id, style, Vec::new())
    }

    pub fn new_with_layout_id(
        layout_id: LayoutNodeId,
        node_id: &'a NodeId,
        style: &'a ComputedStyle,
        children: Vec<LayoutNodeId>,
    ) -> Self {
        Self {
            layout_id,
            node_id: Some(*node_id),
            style: CopiedStyle::Defined(style),
            children,
        }
    }

    pub fn new_anonymous_node_with_layout_id(
        layout_id: LayoutNodeId,
        buffer: Vec<LayoutNodeId>,
        style: &'a ComputedStyle,
    ) -> Self {
        let mut inherited = style.inherited_subset();
        inherited.display = OutsideDisplay::Block.into();

        Self {
            layout_id,
            node_id: None,
            style: CopiedStyle::Anonymous(Box::new(inherited)),
            children: buffer,
        }
    }
}
