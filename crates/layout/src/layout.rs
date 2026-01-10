use std::sync::Arc;

use cosmic_text::Buffer;
use html_syntax::dom::NodeId;

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

    /// The resolved padding values
    pub resolved_padding: SideOffset,

    /// Optional text buffer for rendered text
    pub text_buffer: Option<Arc<Buffer>>,

    /// Child layout nodes
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    /// Calculate the total height including margins and padding
    pub fn margin_box_height(&self) -> f32 {
        self.dimensions.height + self.resolved_margin.vertical() + self.resolved_padding.vertical()
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
