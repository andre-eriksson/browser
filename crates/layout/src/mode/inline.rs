use css_style::{ComputedStyle, StyledNode};
use html_dom::NodeId;

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
        parent_style: &'node ComputedStyle,
        nodes: &'node [&StyledNode],
        image_ctx: &ImageContext,
    ) -> Vec<InlineItem<'node>> {
        let mut raw_items = Vec::with_capacity(nodes.len() * 2);

        for node in nodes {
            if collect(parent_style, node, &mut raw_items, image_ctx).is_err() {
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

                    if let Some(mut layout_node) = LayoutEngine::layout_node(node, &mut block_ctx, text_ctx) {
                        let total_width = layout_node.dimensions.width + padding.horizontal() + border.horizontal();

                        let alignment = &style.text_align;
                        let writing_mode = &style.writing_mode;
                        text_ctx.last_text_align = *alignment;
                        text_ctx.last_writing_mode = *writing_mode;

                        if line.line_box.width + total_width > inline_ctx.containing_block.width
                            && line.line_box.width > 0.0
                        {
                            line.finish_line_with_decorations(&mut inline_layout_ctx, text_ctx, ctx.float_ctx(), None);
                        }

                        let ascent = layout_node.dimensions.height + margin.top + margin.bottom;

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
}
