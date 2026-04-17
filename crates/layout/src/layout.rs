use std::sync::Arc;

use cosmic_text::Buffer;
use css_style::{Color4f, ComputedStyle, Position, StyledNode};
use css_values::cursor::Cursor;
use html_dom::NodeId;

use crate::{
    ImageContext,
    builder::NodeBuilder,
    float::FloatContext,
    mode::block::BlockContext,
    position::PositionContext,
    primitives::{Rect, SideOffset},
};

#[derive(Debug, Clone)]
pub struct BorderColor {
    pub top: Color4f,
    pub right: Color4f,
    pub bottom: Color4f,
    pub left: Color4f,
}

impl Default for BorderColor {
    fn default() -> Self {
        Self {
            top: Color4f::BLACK,
            right: Color4f::BLACK,
            bottom: Color4f::BLACK,
            left: Color4f::BLACK,
        }
    }
}

/// Color properties extracted for rendering
#[derive(Debug, Clone)]
pub struct LayoutColors {
    /// The background color of the layout node
    pub background_color: Color4f,

    /// Text color of the layout node
    pub color: Color4f,

    /// Border color of the layout node
    pub border_color: BorderColor,
}

impl Default for LayoutColors {
    fn default() -> Self {
        Self {
            background_color: Color4f::TRANSPARENT,
            color: Color4f::BLACK,
            border_color: BorderColor::default(),
        }
    }
}

impl LayoutColors {
    /// Creates colors for a text node using only the inherited foreground color.
    /// Background and border are transparent since those come from InlineDecoration.
    pub fn text_only(color: Color4f) -> Self {
        Self {
            background_color: Color4f::TRANSPARENT,
            color,
            border_color: BorderColor::default(),
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
    pub node_id: NodeId,
    pub border: SideOffset,
    pub children: Vec<Self>,
    pub colors: LayoutColors,
    pub cursor: Cursor,
    pub dimensions: Rect,
    pub image_data: Option<ImageData>,
    pub is_height_auto: bool,
    pub margin: SideOffset,
    pub padding: SideOffset,
    pub position: Position,
    pub text_buffer: Option<Arc<Buffer>>,
}

impl LayoutNode {
    pub fn builder(node_id: NodeId) -> NodeBuilder {
        NodeBuilder::new(node_id)
    }
}

/// The root of the layout tree containing all layout nodes
#[derive(Debug, Clone, Default)]
pub struct LayoutTree {
    /// The root layout nodes
    pub root_nodes: Vec<LayoutNode>,

    /// The total content height of the layout tree
    pub content_height: f32,

    /// The total content width of the layout tree
    pub content_width: f32,
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

    fn resolve_in_node<'nodes>(collected: &mut Vec<&'nodes LayoutNode>, node: &'nodes LayoutNode, x: f32, y: f32) {
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
#[derive(Debug)]
pub struct LayoutContext<'layout> {
    containing_block: Rect,
    positioned_containing_block: Rect,
    deferred: bool,
    position_ctx: &'layout mut PositionContext,
    float_ctx: FloatContext,
    image_ctx: &'layout ImageContext,
    pub block_cursor: BlockContext,
}

impl<'layout> LayoutContext<'layout> {
    /// Creates a new LayoutContext with the given containing block
    pub(crate) fn new(
        containing_block: Rect,
        image_ctx: &'layout ImageContext,
        position_ctx: &'layout mut PositionContext,
    ) -> Self {
        Self {
            containing_block,
            positioned_containing_block: containing_block,
            deferred: false,
            position_ctx,
            float_ctx: FloatContext::new(),
            image_ctx,
            block_cursor: BlockContext { y: 0.0 },
        }
    }

    /// Creates a new LayoutContext for deferred layout, which will be used for elements that are
    /// laid out in a second pass after the initial layout has completed.
    pub(crate) fn deferred(
        containing_block: Rect,
        positioned_containing_block: Rect,
        image_ctx: &'layout ImageContext,
        position_ctx: &'layout mut PositionContext,
    ) -> Self {
        Self {
            containing_block,
            positioned_containing_block,
            deferred: true,
            position_ctx,
            float_ctx: FloatContext::new(),
            image_ctx,
            block_cursor: BlockContext { y: 0.0 },
        }
    }

    /// Creates a child context with the specified containing block, inheriting
    /// the image and position contexts.
    pub(crate) fn child_context(&mut self, containing_block: Rect) -> LayoutContext<'_> {
        LayoutContext::new(containing_block, self.image_ctx, &mut *self.position_ctx)
    }

    /// Returns the containing block rect
    pub const fn containing_block(&self) -> Rect {
        self.containing_block
    }

    /// Returns the nearest positioned ancestor containing block used by absolute positioning.
    pub const fn positioned_containing_block(&self) -> Rect {
        self.positioned_containing_block
    }

    pub const fn is_deferred(&self) -> bool {
        self.deferred
    }

    pub fn position_ctx(&mut self) -> &mut PositionContext {
        self.position_ctx
    }

    pub fn float_ctx(&mut self) -> &mut FloatContext {
        &mut self.float_ctx
    }

    pub fn float_ctx_ref(&self) -> &FloatContext {
        &self.float_ctx
    }

    pub fn image_ctx(&self) -> &ImageContext {
        self.image_ctx
    }

    /// Sets the nearest positioned ancestor containing block used by absolute positioning.
    pub const fn set_positioned_containing_block(&mut self, rect: Rect) {
        self.positioned_containing_block = rect;
    }
}
