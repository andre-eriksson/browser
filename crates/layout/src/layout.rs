use std::sync::Arc;

use cosmic_text::Buffer;
use html_dom::NodeId;

use crate::primitives::{Color4f, Rect, SideOffset};

/// Color properties extracted for rendering
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutColors {
    /// The background color of the layout node
    pub background_color: Color4f,

    /// Text color of the layout node
    pub color: Color4f,
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

    /// The collapsed top margin value
    pub collapsed_margin_top: f32,

    /// The collapsed bottom margin value
    pub collapsed_margin_bottom: f32,

    /// The resolved padding values
    pub resolved_padding: SideOffset,

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
            collapsed_margin_top: 0.0,
            collapsed_margin_bottom: 0.0,
            resolved_padding: SideOffset::default(),
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

/// Context passed down during layout computation
#[derive(Debug, Clone, Default)]
pub struct LayoutContext {
    /// The containing block's content rect (where children are positioned)
    pub containing_block: Rect,

    /// The parent node's resolved padding top value
    pub parent_padding_top: f32,

    /// Whether the current node is the first child of its parent
    pub is_first_child: bool,
}
