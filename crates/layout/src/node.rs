use std::sync::Arc;

use cosmic_text::Buffer;

use css_style::Position;
use css_values::cursor::Cursor;
use html_dom::NodeId;

use crate::{ImageData, LayoutColors, Rect, primitives::SideOffset};

/// A node in the layout tree representing a rendered element
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub node_id: Option<NodeId>,
    pub border: SideOffset,
    pub children: Vec<Self>,
    pub colors: LayoutColors,
    pub cursor: Cursor,
    pub dimensions: Rect,
    pub image_data: Option<ImageData>,
    pub is_height_auto: bool,
    pub position: Position,
    pub text_buffer: Option<Arc<Buffer>>,
}

impl LayoutNode {
    #[must_use]
    pub fn builder(node_id: Option<NodeId>) -> NodeBuilder {
        NodeBuilder::new(node_id)
    }
}

/// Builder pattern for constructing a `LayoutNode`.
#[derive(Debug, Clone)]
pub struct NodeBuilder {
    pub layout_node: LayoutNode,
}

impl NodeBuilder {
    pub fn new(node_id: Option<NodeId>) -> Self {
        Self {
            layout_node: LayoutNode {
                node_id,
                dimensions: Rect::default(),
                colors: LayoutColors::default(),
                cursor: Cursor::default(),
                border: SideOffset::default(),
                position: Position::Static,
                text_buffer: None,
                image_data: None,
                children: Vec::new(),
                is_height_auto: false,
            },
        }
    }

    pub const fn border(mut self, border: SideOffset) -> Self {
        self.layout_node.border = border;
        self
    }

    pub fn children(mut self, children: Vec<LayoutNode>) -> Self {
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

    pub const fn height_auto(mut self, is_height_auto: bool) -> Self {
        self.layout_node.is_height_auto = is_height_auto;
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
