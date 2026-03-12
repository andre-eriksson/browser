use std::sync::Arc;

use cosmic_text::Buffer;
use css_style::{Color4f, ComputedStyle, StyledNode};
use css_values::cursor::Cursor;
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

#[derive(Debug, Clone)]
pub struct ImageData {
    /// Image source URL for `<img>` elements
    pub image_src: String,

    /// The pre-resolved Vary string for this image's cache entry, computed from
    /// the response headers at fetch time. This allows exact disk cache lookups
    /// without needing the full `HeaderMap`.
    pub vary_key: String,

    /// Whether this image node is using placeholder dimensions and should be
    /// updated to the intrinsic image size once the image has been decoded.
    pub image_needs_intrinsic_size: bool,
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

    /// The cursor style for this layout node
    pub cursor: Cursor,

    /// The resolved margin values
    pub resolved_margin: SideOffset,

    /// The resolved padding values
    pub resolved_padding: SideOffset,

    /// The resolved border widths
    pub resolved_border: SideOffset,

    /// Optional text buffer for rendered text
    pub text_buffer: Option<Arc<Buffer>>,

    /// Optional image data for rendered images
    pub image_data: Option<ImageData>,

    /// Child layout nodes
    pub children: Vec<LayoutNode>,

    /// Whether this node's height is determined by its content (e.g. for block elements)
    pub is_height_auto: bool,
}

impl LayoutNode {
    /// Creates a new LayoutNode with default values
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            dimensions: Rect::default(),
            colors: LayoutColors::default(),
            cursor: Cursor::default(),
            resolved_margin: SideOffset::default(),
            resolved_padding: SideOffset::default(),
            resolved_border: SideOffset::default(),
            text_buffer: None,
            image_data: None,
            children: Vec::new(),
            is_height_auto: false,
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

    fn resolve_in_node<'a>(collected: &mut Vec<&'a LayoutNode>, node: &'a LayoutNode, x: f32, y: f32) {
        if node.dimensions.contains_point(x, y) {
            for child in &node.children {
                Self::resolve_in_node(collected, child, x, y);
            }
            collected.push(node);
        }
    }

    /// Collect the `NodeId` of every image node whose `image_src` matches `url`.
    ///
    /// An image node is any [`LayoutNode`] that has `image_data.image_src == url`.
    /// There may be more than one if the same image appears multiple times on the page.
    pub fn find_image_nodes_by_src(&self, url: &str) -> Vec<NodeId> {
        let mut result = Vec::new();
        for root in &self.root_nodes {
            Self::collect_image_nodes(root, url, &mut result);
        }
        result
    }

    fn collect_image_nodes(node: &LayoutNode, url: &str, out: &mut Vec<NodeId>) {
        if let Some(ref img) = node.image_data
            && img.image_src == url
        {
            out.push(node.node_id);
        }

        for child in &node.children {
            Self::collect_image_nodes(child, url, out);
        }
    }

    /// Finds the path to the layout node corresponding to the given `NodeId`, if it exists.
    pub fn find_path(&self, node_id: NodeId) -> Option<Vec<usize>> {
        for (idx, root) in self.root_nodes.iter().enumerate() {
            if let Some(mut path) = Self::find_path_in_node(root, node_id) {
                path.insert(0, idx);
                return Some(path);
            }
        }

        None
    }

    fn find_path_in_node(node: &LayoutNode, node_id: NodeId) -> Option<Vec<usize>> {
        if node.node_id == node_id {
            return Some(vec![]);
        }

        for (idx, child) in node.children.iter().enumerate() {
            if let Some(mut path) = Self::find_path_in_node(child, node_id) {
                path.insert(0, idx);
                return Some(path);
            }
        }

        None
    }

    /// Retrieves a reference to the layout node at the specified path, if it exists.
    pub fn node_at(&self, path: &[usize]) -> Option<&LayoutNode> {
        if path.is_empty() {
            return None;
        }

        let mut current = self.root_nodes.get(path[0]);
        for &idx in &path[1..] {
            current = match current {
                None => return current,
                Some(node) => node.children.get(idx),
            };
        }
        current
    }

    /// Retrieves a mutable reference to the layout node at the specified path, if it exists.
    pub fn node_at_mut(&mut self, path: &[usize]) -> Option<&mut LayoutNode> {
        if path.is_empty() {
            return None;
        }

        let mut current = self.root_nodes.get_mut(path[0]);
        for &idx in &path[1..] {
            current = match current {
                None => return current,
                Some(node) => node.children.get_mut(idx),
            };
        }
        current
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
