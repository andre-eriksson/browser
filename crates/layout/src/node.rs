use std::sync::Arc;

use cosmic_text::Buffer;

use css_display::LayoutNodeId;
use css_style::Position;
use css_values::cursor::Cursor;
use html_dom::NodeId;

use crate::{ImageData, LayoutColors, Margin, Rect, primitives::SideOffset};

/// A node in the layout tree representing a rendered element
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub block_formatting_context: bool,
    pub border: SideOffset,
    pub children: Vec<LayoutNodeId>,
    pub colors: LayoutColors,
    pub cursor: Cursor,
    pub dimensions: Rect,
    pub image_data: Option<ImageData>,
    pub layout_id: LayoutNodeId,
    pub margin: Margin,
    pub node_id: Option<NodeId>,
    pub padding: SideOffset,
    pub position: Position,
    pub text_buffer: Option<Arc<Buffer>>,
}

impl LayoutNode {
    #[must_use]
    pub fn builder(layout_id: LayoutNodeId) -> NodeBuilder {
        NodeBuilder::new(layout_id)
    }

    pub fn insert_child(&mut self, child_id: LayoutNodeId) {
        self.children.push(child_id);
    }
}

/// Builder pattern for constructing a `LayoutNode`.
#[derive(Debug, Clone)]
pub struct NodeBuilder {
    pub layout_node: LayoutNode,
}

impl NodeBuilder {
    pub fn new(layout_id: LayoutNodeId) -> Self {
        Self {
            layout_node: LayoutNode {
                block_formatting_context: false,
                border: SideOffset::default(),
                children: Vec::new(),
                colors: LayoutColors::default(),
                cursor: Cursor::default(),
                dimensions: Rect::default(),
                image_data: None,
                layout_id,
                margin: Margin::default(),
                node_id: None,
                padding: SideOffset::default(),
                position: Position::Static,
                text_buffer: None,
            },
        }
    }

    pub const fn block_formatting_context(mut self, block_formatting_context: bool) -> Self {
        self.layout_node.block_formatting_context = block_formatting_context;
        self
    }

    pub const fn border(mut self, border: SideOffset) -> Self {
        self.layout_node.border = border;
        self
    }

    pub fn children(mut self, children: Vec<LayoutNodeId>) -> Self {
        self.layout_node.children = children;
        self
    }

    pub const fn colors(mut self, colors: LayoutColors) -> Self {
        self.layout_node.colors = colors;
        self
    }

    pub const fn cursor(mut self, cursor: Cursor) -> Self {
        self.layout_node.cursor = cursor;
        self
    }

    pub const fn dimensions(mut self, rect: Rect) -> Self {
        self.layout_node.dimensions = rect;
        self
    }

    pub fn image_data(mut self, image_data: ImageData) -> Self {
        self.layout_node.image_data = Some(image_data);
        self
    }

    pub const fn margin(mut self, margin: Margin) -> Self {
        self.layout_node.margin = margin;
        self
    }

    pub const fn node_id(mut self, node_id: NodeId) -> Self {
        self.layout_node.node_id = Some(node_id);
        self
    }

    pub const fn maybe_node_id(mut self, maybe_node_id: Option<NodeId>) -> Self {
        self.layout_node.node_id = maybe_node_id;
        self
    }

    pub const fn padding(mut self, padding: SideOffset) -> Self {
        self.layout_node.padding = padding;
        self
    }

    pub const fn position(mut self, position: Position) -> Self {
        self.layout_node.position = position;
        self
    }

    pub fn text_buffer(mut self, text_buffer: Arc<Buffer>) -> Self {
        self.layout_node.text_buffer = Some(text_buffer);
        self
    }

    pub fn build(self) -> LayoutNode {
        self.layout_node
    }
}
