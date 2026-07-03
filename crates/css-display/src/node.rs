use std::{fmt::Debug, ops::Deref};

use css_style::ComputedStyle;
use css_values::display::OutsideDisplay;
use html_dom::NodeId;

use crate::tree::ChildFormattingContext;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[derive(Clone)]
pub struct BoxNode<'a> {
    pub parent_id: Option<LayoutNodeId>,
    pub layout_id: LayoutNodeId,
    pub node_id: Option<NodeId>,
    pub style: CopiedStyle<'a>,
    pub children: Vec<LayoutNodeId>,
}

impl<'a> BoxNode<'a> {
    pub fn new(
        parent_id: Option<LayoutNodeId>,
        layout_id: LayoutNodeId,
        node_id: &'a NodeId,
        style: &'a ComputedStyle,
        children: Vec<LayoutNodeId>,
    ) -> Self {
        Self {
            parent_id,
            layout_id,
            node_id: Some(*node_id),
            style: CopiedStyle::Defined(style),
            children,
        }
    }

    pub fn new_anonymous_node(
        parent_id: Option<LayoutNodeId>,
        layout_id: LayoutNodeId,
        parent_style: &'a ComputedStyle,
        children: Vec<LayoutNodeId>,
        cfc: ChildFormattingContext,
    ) -> Self {
        let mut inherited = parent_style.inherited_subset();
        if matches!(cfc, ChildFormattingContext::Block) {
            inherited.display = OutsideDisplay::Block.into();
        }

        Self {
            parent_id,
            layout_id,
            node_id: None,
            style: CopiedStyle::Anonymous(Box::new(inherited)),
            children,
        }
    }
}

impl<'a> Debug for BoxNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxNode")
            .field("parent_id", &self.parent_id)
            .field("layout_id", &self.layout_id)
            .field("node_id", &self.node_id)
            .field("is_anonymous", &matches!(self.style, CopiedStyle::Anonymous(_)))
            .field("children", &self.children)
            .finish()
    }
}
