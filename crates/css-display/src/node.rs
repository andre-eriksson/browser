use std::ops::Deref;

use css_style::ComputedStyle;
use css_values::display::OutsideDisplay;
use html_dom::NodeId;

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
    pub node_id: Option<NodeId>,
    pub style: CopiedStyle<'a>,
    pub children: Vec<BoxNode<'a>>,
}

impl<'a> BoxNode<'a> {
    pub fn new(node_id: &'a NodeId, style: &'a ComputedStyle, children: Vec<BoxNode<'a>>) -> Self {
        Self {
            node_id: Some(*node_id),
            style: CopiedStyle::Defined(style),
            children,
        }
    }

    pub fn new_anonymous_node(buffer: Vec<BoxNode<'a>>, style: &'a ComputedStyle) -> Self {
        let mut inherited = style.inherited_subset();
        inherited.display = OutsideDisplay::Block.into();

        Self {
            node_id: None,
            style: CopiedStyle::Anonymous(Box::new(inherited)),
            children: buffer,
        }
    }
}
