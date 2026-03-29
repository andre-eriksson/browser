use css_style::{ComputedDimension, ComputedStyle, StyledNode};
use html_dom::NodeId;

use crate::{
    LayoutEngine, LayoutNode, Rect, SideOffset, TextContext,
    context::ImageContext,
    float::FloatContext,
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
pub(crate) struct InlineDecoration<'node> {
    id: NodeId,
    start_x: f32,
    end_x: f32,
    style: &'node ComputedStyle,
    padding: SideOffset,
    border: SideOffset,
}

/// State for an inline box that has been opened but not yet closed.
#[derive(Debug, Clone)]
pub(crate) struct ActiveInlineBox<'node> {
    id: NodeId,
    style: &'node ComputedStyle,
    start_x: f32,
    margin: SideOffset,
    padding: SideOffset,
    border: SideOffset,
}

/// Context passed around during inline layout, allowing helper functions to update the current line box, emit positioned layout nodes, and track active inline boxes for decoration purposes.
pub(crate) struct InlineLayoutContext<'a, 'node> {
    pub current_y: &'a mut f32,
    pub start_x: f32,
    pub available_width: f32,
    pub float_context: &'a FloatContext,
    pub nodes: &'a mut Vec<LayoutNode>,
    pub inline_box_stack: &'a mut Vec<ActiveInlineBox<'node>>,
}

pub struct InlineLayout;

impl InlineLayout {
    /// Collects inline items from the given styled nodes, recursively traversing into inline children but
    /// returning an error if it encounters a block-level element (which should be handled by the block layout instead).
    /// The resulting flat list of inline items is then canonicalised by collapsing whitespace
    /// according to the CSS `white-space` property of each text run and stripping leading/trailing whitespace from lines.
    pub fn collect_inline_items_from_nodes<'node>(
        parent_style: &'node ComputedStyle,
        nodes: &'node [StyledNode],
        image_ctx: &ImageContext,
    ) -> Vec<InlineItem<'node>> {
        let mut raw_items = Vec::with_capacity(nodes.len() * 2);

        for node in nodes {
            let result = collect(parent_style, node, &mut raw_items, image_ctx);

            if result.is_err() {
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
        text_ctx: &mut TextContext,
        float_ctx: &FloatContext,
        available_width: f32,
        start_x: f32,
        start_y: f32,
        image_ctx: &ImageContext,
    ) -> (Vec<LayoutNode>, f32) {
        let mut nodes = Vec::new();
        let mut current_y = start_y;
        let mut line = LineBoxBuilder::new(start_x, start_y);

        let mut inline_box_stack: Vec<ActiveInlineBox> = Vec::new();

        for item in items {
            match item {
                InlineItem::TextRun(text) => {
                    layout_text(
                        InlineLayoutContext {
                            available_width,
                            float_context: float_ctx,
                            current_y: &mut current_y,
                            start_x,
                            nodes: &mut nodes,
                            inline_box_stack: &mut inline_box_stack,
                        },
                        text_ctx,
                        &mut line,
                        text,
                    );
                }
                InlineItem::InlineBoxStart { id, style } => {
                    line.open_inline_box(&mut inline_box_stack, text_ctx, *id, style);
                }
                InlineItem::InlineBoxEnd { id } => {
                    line.close_inline_box(&mut inline_box_stack, *id);
                }
                InlineItem::InlineFlowRoot { node, style } => {
                    let (margin, padding, border) = PropertyResolver::resolve_box_model(style);

                    let desired_width = match style.width {
                        ComputedDimension::Percentage(f) => available_width * f,
                        _ => style.intrinsic_width,
                    };

                    let mut block_ctx = LayoutContext::new(Rect::new(0.0, 0.0, desired_width, 0.0));

                    if let Some(mut layout_node) = LayoutEngine::layout_node(node, &mut block_ctx, text_ctx, image_ctx)
                    {
                        let total_width = layout_node.dimensions.width + padding.horizontal() + border.horizontal();

                        let alignment = &style.text_align;
                        let writing_mode = &style.writing_mode;
                        text_ctx.last_text_align = *alignment;
                        text_ctx.last_writing_mode = *writing_mode;

                        if line.line_box.width + total_width > available_width && line.line_box.width > 0.0 {
                            line.finish_line_with_decorations(
                                &mut InlineLayoutContext {
                                    available_width,
                                    float_context: float_ctx,
                                    current_y: &mut current_y,
                                    start_x,
                                    inline_box_stack: &mut inline_box_stack,
                                    nodes: &mut nodes,
                                },
                                text_ctx,
                                None,
                            );
                        }

                        let ascent = layout_node.dimensions.height + margin.top + margin.bottom;

                        layout_node.margin = margin;

                        line.line_box.add(layout_node, ascent, 0.0);
                    }
                }
                InlineItem::Image(img) => {
                    layout_image(
                        InlineLayoutContext {
                            available_width,
                            float_context: float_ctx,
                            current_y: &mut current_y,
                            start_x,
                            inline_box_stack: &mut inline_box_stack,
                            nodes: &mut nodes,
                        },
                        img,
                        text_ctx,
                        &mut line,
                        image_ctx,
                    );
                }
                InlineItem::Break { line_height_px } => {
                    line.finish_line_with_decorations(
                        &mut InlineLayoutContext {
                            available_width,
                            float_context: float_ctx,
                            current_y: &mut current_y,
                            start_x,
                            inline_box_stack: &mut inline_box_stack,
                            nodes: &mut nodes,
                        },
                        text_ctx,
                        Some(*line_height_px),
                    );
                }
            }
        }

        line.close_active_decorations(&mut inline_box_stack);

        let (line_nodes, h) = line.line_box.finish(
            float_ctx,
            start_x,
            available_width,
            &text_ctx.last_text_align,
            &text_ctx.last_writing_mode,
        );
        nodes.extend(line_nodes);
        let total_height = current_y + h - start_y;

        (nodes, total_height)
    }
}
