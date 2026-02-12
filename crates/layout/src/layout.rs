use std::sync::Arc;

use cosmic_text::Buffer;
use css_style::{Color4f, ComputedStyle, StyledNode};
use html_dom::NodeId;

use crate::{
    mode::block::BlockCursor,
    primitives::{Rect, SideOffset},
};

#[derive(Debug, Clone, Default)]
pub struct BorderColor {
    pub top: Color4f,
    pub right: Color4f,
    pub bottom: Color4f,
    pub left: Color4f,
}

/// Color properties extracted for rendering
#[derive(Debug, Clone, Default)]
pub struct LayoutColors {
    /// The background color of the layout node
    pub background_color: Color4f,

    /// Text color of the layout node
    pub color: Color4f,

    /// Border color of the layout node
    pub border_color: BorderColor,
}

impl LayoutColors {
    pub fn inline(color: Color4f) -> Self {
        Self {
            color,
            ..Default::default()
        }
    }
}

impl From<&ComputedStyle> for LayoutColors {
    fn from(style: &ComputedStyle) -> Self {
        Self {
            background_color: style.background_color,
            color: style.color,
            border_color: BorderColor {
                top: style.border_top_color,
                right: style.border_right_color,
                bottom: style.border_bottom_color,
                left: style.border_left_color,
            },
        }
    }
}

impl From<&Box<ComputedStyle>> for LayoutColors {
    fn from(style: &Box<ComputedStyle>) -> Self {
        Self::from(style.as_ref())
    }
}

impl From<&StyledNode> for LayoutColors {
    fn from(styled_node: &StyledNode) -> Self {
        Self::from(&styled_node.style)
    }
}

/// A node in the layout tree representing a rendered element
#[derive(Debug, Clone)]
pub struct LayoutNode {
    /// The associated DOM node ID
    pub node_id: NodeId,

    /// The dimensions and position of the layout node
    pub dimensions: Rect,

    /// The color properties for rendering
    pub colors: LayoutColors,

    /// The resolved margin values
    pub resolved_margin: SideOffset,

    /// The resolved padding values
    pub resolved_padding: SideOffset,

    /// The resolved border widths
    pub resolved_border: SideOffset,

    /// Optional text buffer for rendered text
    pub text_buffer: Option<Arc<Buffer>>,

    /// Child layout nodes
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    /// Creates a new LayoutNode with default values
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            dimensions: Rect::default(),
            colors: LayoutColors::default(),
            resolved_margin: SideOffset::default(),
            resolved_padding: SideOffset::default(),
            resolved_border: SideOffset::default(),
            text_buffer: None,
            children: Vec::new(),
        }
    }
}

/// The root of the layout tree containing all layout nodes
#[derive(Debug, Clone, Default)]
pub struct LayoutTree {
    /// The root layout nodes
    pub root_nodes: Vec<LayoutNode>,

    /// The total content height of the layout tree
    pub content_height: f32,
}

impl LayoutTree {
    /// Resolves the layout node at the given (x, y) coordinates
    pub fn resolve(&self, x: f32, y: f32) -> Vec<&LayoutNode> {
        let mut collected = Vec::new();
        for node in &self.root_nodes {
            Self::resolve_in_node(&mut collected, node, x, y);
        }
        collected
    }

    fn resolve_in_node<'a>(
        collected: &mut Vec<&'a LayoutNode>,
        node: &'a LayoutNode,
        x: f32,
        y: f32,
    ) {
        if node.dimensions.contains_point(x, y) {
            for child in &node.children {
                Self::resolve_in_node(collected, child, x, y);
            }
            collected.push(node);
        }
    }
}

/// Context passed down during layout computation
#[derive(Debug, Clone, Default)]
pub struct LayoutContext {
    /// The containing block's content rect (where children are positioned)
    containing_block: Rect,

    /// The current block cursor position
    pub block_cursor: BlockCursor,
}

impl LayoutContext {
    /// Creates a new LayoutContext with the given containing block
    pub fn new(containing_block: Rect) -> Self {
        Self {
            containing_block,
            block_cursor: BlockCursor::from(containing_block.y),
        }
    }

    /// Returns the containing block rect
    pub fn containing_block(&self) -> Rect {
        self.containing_block
    }
}
