use css_style::{ComputedDimension, ComputedStyle, StyledNode};
use html_dom::{DocumentRoot, NodeId};

use crate::{
    LayoutEngine, LayoutNode, Rect, SideOffset, TextContext,
    context::ImageContext,
    layout::LayoutContext,
    mode::inline::{
        collection::{InlineItem, collect},
        image::layout_image,
        line::LineBoxBuilder,
        text::layout_text,
        whitespace::canonicalize_whitespace,
    },
    resolver::PropertyResolver,
};

mod collection;
mod image;
mod line;
mod text;
mod whitespace;

/// Tracks an inline box decoration (background, border, padding) that needs to
/// be emitted as a `LayoutNode` once the line is finished and final positions
/// are known.
#[derive(Debug, Clone)]
pub struct InlineDecoration<'node> {
    id: NodeId,
    start_x: f32,
    end_x: f32,
    style: &'node ComputedStyle,
    padding: SideOffset,
    border: SideOffset,
}

/// State for an inline box that has been opened but not yet closed.
#[derive(Debug, Clone)]
pub struct ActiveInlineBox<'node> {
    id: NodeId,
    style: &'node ComputedStyle,
    start_x: f32,
    margin: SideOffset,
    padding: SideOffset,
    border: SideOffset,
}

/// Context passed around during inline layout, allowing helper functions to update the current line box, emit positioned layout nodes, and track active inline boxes for decoration purposes.
#[derive(Debug, Default)]
pub struct InlineLayoutContext<'node> {
    pub current_y: f32,
    pub start_x: f32,
    pub available_width: f32,
    pub nodes: Vec<LayoutNode>,
    pub inline_box_stack: Vec<ActiveInlineBox<'node>>,
}

#[derive(Debug, Clone, Copy)]
pub struct InlineContext {
    containing_block: Rect,
}

impl InlineContext {
    pub const fn new(containing_block: Rect) -> Self {
        Self { containing_block }
    }
}

pub struct InlineLayout;

impl InlineLayout {
    /// Collects inline items from the given styled nodes, recursively traversing into inline children but
    /// returning an error if it encounters a block-level element (which should be handled by the block layout instead).
    /// The resulting flat list of inline items is then canonicalised by collapsing whitespace
    /// according to the CSS `white-space` property of each text run and stripping leading/trailing whitespace from lines.
    pub fn collect_inline_items_from_nodes<'node>(
        dom_tree: &DocumentRoot,
        parent_style: &'node ComputedStyle,
        nodes: &'node [&StyledNode],
        image_ctx: &ImageContext,
    ) -> Vec<InlineItem<'node>> {
        let mut raw_items = Vec::with_capacity(nodes.len() * 2);

        for node in nodes {
            if collect(dom_tree, parent_style, node, &mut raw_items, image_ctx).is_err() {
                break;
            }
        }

        canonicalize_whitespace(&mut raw_items);
        raw_items
    }

    /// The main entry point for inline layout: given a list of styled nodes that
    /// contribute to an inline formatting context, first collect them into a
    /// flat list of `InlineItem`s, then measure and position those items into
    /// one or more `LineBox`es according to the available width and text alignment,
    /// finally returning the positioned `LayoutNode`s and total height of the laid-out lines.
    pub fn layout(
        dom_tree: &DocumentRoot,
        items: &[InlineItem],
        ctx: &mut LayoutContext,
        text_ctx: &mut TextContext,
        inline_ctx: InlineContext,
    ) -> (Vec<LayoutNode>, Rect) {
        let mut line = LineBoxBuilder::new(inline_ctx.containing_block.x, inline_ctx.containing_block.y);

        let mut inline_layout_ctx = InlineLayoutContext {
            available_width: inline_ctx.containing_block.width,
            current_y: inline_ctx.containing_block.y,
            start_x: inline_ctx.containing_block.x,
            nodes: Vec::new(),
            inline_box_stack: Vec::new(),
        };

        for item in items {
            match item {
                InlineItem::TextRun(text) => {
                    layout_text(&mut inline_layout_ctx, ctx.float_ctx(), text_ctx, &mut line, text);
                }
                InlineItem::InlineBoxStart { id, style } => {
                    line.open_inline_box(&mut inline_layout_ctx.inline_box_stack, text_ctx, *id, style);
                }
                InlineItem::InlineBoxEnd { id } => {
                    line.close_inline_box(&mut inline_layout_ctx.inline_box_stack, *id);
                }
                InlineItem::InlineFlowRoot { node, style } => {
                    let (margin, padding, border) = PropertyResolver::resolve_box_model(style);

                    let img_ctx = ctx.image_ctx().clone();
                    let mut block_ctx = LayoutContext::new(
                        Rect::new(
                            inline_ctx.containing_block.x,
                            inline_ctx.containing_block.y,
                            inline_ctx.containing_block.width,
                            inline_ctx.containing_block.height,
                        ),
                        &img_ctx,
                        ctx.position_ctx(),
                    );

                    if let Some(mut layout_node) = LayoutEngine::layout_node(dom_tree, node, &mut block_ctx, text_ctx) {
                        if style.width == ComputedDimension::Auto {
                            layout_node.dimensions.width =
                                InlineLayout::auto_inline_flow_root_width(&layout_node, padding, border)
                                    .min(layout_node.dimensions.width);
                        }

                        let total_width = layout_node.dimensions.width + margin.horizontal();
                        let available_line_width = line
                            .line_box
                            .available_width(ctx.float_ctx_ref(), inline_layout_ctx.available_width);

                        let alignment = &style.text_align;
                        let writing_mode = &style.writing_mode;
                        text_ctx.last_text_align = *alignment;
                        text_ctx.last_writing_mode = *writing_mode;

                        if line.line_box.width + total_width > available_line_width && line.line_box.width > 0.0 {
                            line.finish_line_with_decorations(&mut inline_layout_ctx, text_ctx, ctx.float_ctx(), None);
                        }

                        let ascent = layout_node.dimensions.height + margin.vertical();

                        layout_node.margin = margin;

                        line.line_box.add(layout_node, ascent, 0.0);
                    }
                }
                InlineItem::Image(img) => {
                    layout_image(&mut inline_layout_ctx, img, text_ctx, ctx, &mut line);
                }
                InlineItem::Break { line_height_px } => {
                    line.finish_line_with_decorations(
                        &mut inline_layout_ctx,
                        text_ctx,
                        ctx.float_ctx(),
                        Some(*line_height_px),
                    );
                }
            }
        }

        line.close_active_decorations(&mut inline_layout_ctx.inline_box_stack);

        let (line_nodes, h) = line.line_box.finish(
            ctx.float_ctx(),
            inline_ctx.containing_block.x,
            inline_ctx.containing_block.width,
            &text_ctx.last_text_align,
            &text_ctx.last_writing_mode,
        );
        inline_layout_ctx.nodes.extend(line_nodes);
        let total_height = inline_layout_ctx.current_y + h - inline_ctx.containing_block.y;

        (
            inline_layout_ctx.nodes,
            Rect::new(
                inline_ctx.containing_block.x,
                inline_ctx.containing_block.y,
                inline_ctx.containing_block.width,
                total_height,
            ),
        )
    }

    fn auto_inline_flow_root_width(layout_node: &LayoutNode, padding: SideOffset, border: SideOffset) -> f32 {
        let content_left = layout_node.dimensions.x + padding.left + border.left;
        let max_right = layout_node
            .children
            .iter()
            .fold(content_left, |right, child| right.max(InlineLayout::max_right_edge(child)));
        let content_width = (max_right - content_left).max(0.0);

        content_width + padding.horizontal() + border.horizontal()
    }

    fn max_right_edge(node: &LayoutNode) -> f32 {
        let mut right = node.dimensions.x + node.dimensions.width;
        for child in &node.children {
            right = right.max(InlineLayout::max_right_edge(child));
        }

        right
    }
}

#[cfg(test)]
mod tests {
    use html_dom::NodeId;

    use super::*;

    #[test]
    fn auto_inline_flow_root_width_uses_descendant_extent() {
        let nested_text = LayoutNode::builder(NodeId(3))
            .dimensions(Rect::new(13.0, 0.0, 25.0, 12.0))
            .build();
        let inline_child = LayoutNode::builder(NodeId(2))
            .dimensions(Rect::new(13.0, 0.0, 25.0, 12.0))
            .children(vec![nested_text])
            .build();
        let container = LayoutNode::builder(NodeId(1))
            .dimensions(Rect::new(8.0, 0.0, 500.0, 20.0))
            .children(vec![inline_child])
            .build();

        let padding = SideOffset {
            top: 0.0,
            right: 4.0,
            bottom: 0.0,
            left: 4.0,
        };
        let border = SideOffset {
            top: 0.0,
            right: 1.0,
            bottom: 0.0,
            left: 1.0,
        };

        let width = InlineLayout::auto_inline_flow_root_width(&container, padding, border);

        assert!((width - 35.0).abs() < 0.001);
    }
}
