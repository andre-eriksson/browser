use css_display::LayoutNodeId;
use css_style::{ComputedSize, ComputedStyle};
use html_dom::NodeId;

use crate::{
    LayoutInput, LayoutNode, Margin, Rect,
    context::{Geometry, LayoutContext, PositionContext},
    mode::{
        block::{BlockContext, BlockLayout},
        inline::{
            collection::{InlineItem, collect},
            image::layout_image,
            line::LineBoxBuilder,
            text::layout_text,
            whitespace::canonicalize_whitespace,
        },
    },
    primitives::SideOffset,
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
    layout_id: LayoutNodeId,
    node_id: Option<NodeId>,
    start_x: f64,
    end_x: f64,
    style: &'node ComputedStyle,
    padding: SideOffset,
    border: SideOffset,
}

/// State for an inline box that has been opened but not yet closed.
#[derive(Debug, Clone)]
pub struct ActiveInlineBox<'node> {
    layout_id: LayoutNodeId,
    node_id: Option<NodeId>,
    style: &'node ComputedStyle,
    start_x: f64,
    margin: Margin,
    padding: SideOffset,
    border: SideOffset,
}

/// Context passed around during inline layout, allowing helper functions to update the current line box, emit positioned layout nodes, and track active inline boxes for decoration purposes.
#[derive(Debug, Default)]
pub struct InlineLayoutContext<'node> {
    pub current_y: f64,
    pub start_x: f64,
    pub available_width: f64,
    pub ids: Vec<LayoutNodeId>,
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
    pub fn collect_inline_items_from_node<'dom>(
        containing_rect: Rect,
        input: &mut LayoutInput<'dom>,
        parent_style: &'dom ComputedStyle,
        node: &'dom LayoutNodeId,
    ) -> Vec<InlineItem<'dom>> {
        let mut raw_items = Vec::new();

        if collect(containing_rect, input, parent_style, node, &mut raw_items).is_err() {
            return raw_items;
        }

        canonicalize_whitespace(&mut raw_items);

        raw_items
    }

    /// Collects inline items from the given styled nodes, recursively traversing into inline children but
    /// returning an error if it encounters a block-level element (which should be handled by the block layout instead).
    /// The resulting flat list of inline items is then canonicalised by collapsing whitespace
    /// according to the CSS `white-space` property of each text run and stripping leading/trailing whitespace from lines.
    pub fn collect_inline_items_from_nodes<'dom>(
        containing_rect: Rect,
        input: &mut LayoutInput<'dom>,
        parent_style: &'dom ComputedStyle,
        nodes: &'dom [LayoutNodeId],
    ) -> Vec<InlineItem<'dom>> {
        let mut raw_items = Vec::with_capacity(nodes.len() * 2);

        for node in nodes {
            if collect(containing_rect, input, parent_style, node, &mut raw_items).is_err() {
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
        nodes: &mut Vec<Option<LayoutNode>>,
        input: &mut LayoutInput<'_>,
        items: &[InlineItem],
        ctx: &mut LayoutContext,
        position_ctx: &mut PositionContext,
        inline_ctx: InlineContext,
    ) -> (Vec<LayoutNodeId>, Rect) {
        let mut line = LineBoxBuilder::new(
            inline_ctx.containing_block.width,
            inline_ctx.containing_block.x,
            inline_ctx.containing_block.y,
        );

        let mut inline_layout_ctx = InlineLayoutContext {
            available_width: inline_ctx.containing_block.width,
            current_y: inline_ctx.containing_block.y,
            start_x: inline_ctx.containing_block.x,
            ids: Vec::new(),
            inline_box_stack: Vec::new(),
        };

        for item in items {
            match item {
                InlineItem::TextRun(text) => {
                    layout_text(nodes, &mut inline_layout_ctx, ctx.float_ctx(), input.text, &mut line, text);
                }
                InlineItem::InlineBoxStart {
                    layout_id,
                    node_id,
                    style,
                } => {
                    line.open_inline_box(
                        &mut inline_layout_ctx.inline_box_stack,
                        input.text,
                        **layout_id,
                        *node_id,
                        style,
                    );
                }
                InlineItem::InlineBoxEnd { layout_id } => {
                    line.close_inline_box(&mut inline_layout_ctx.inline_box_stack, **layout_id);
                }
                InlineItem::InlineFlowRoot { layout_id, style } => {
                    let box_model = Geometry::resolve_box_model(style, inline_layout_ctx.available_width);
                    let mut ctx = LayoutContext::new(Rect::new(
                        inline_ctx.containing_block.x,
                        inline_ctx.containing_block.y,
                        inline_ctx.containing_block.width,
                        inline_ctx.containing_block.height,
                    ));
                    let mut block_ctx = BlockContext::default();

                    if let Some((id, _)) =
                        BlockLayout::layout(nodes, layout_id, style, input, &mut ctx, position_ctx, &mut block_ctx)
                    {
                        let Some(mut layout_node) = std::mem::take(&mut nodes[id.index()]) else {
                            continue;
                        };

                        if style.width == ComputedSize::Auto {
                            layout_node.dimensions.width = InlineLayout::auto_inline_flow_root_width(
                                nodes,
                                &layout_node,
                                box_model.padding,
                                box_model.border,
                            )
                            .min(layout_node.dimensions.width);
                        }

                        let _total_width = layout_node.dimensions.width
                            + box_model.margin.left.to_px()
                            + box_model.margin.right.to_px();
                        // let available_line_width = line
                        //     .line_box
                        //     .available_width(ctx.float_ctx_ref(), inline_layout_ctx.available_width);

                        let alignment = &style.text_align;
                        let writing_mode = &style.writing_mode;
                        input.text.last_text_align = *alignment;
                        input.text.last_writing_mode = *writing_mode;

                        // if line.line_box.width + total_width > available_line_width && line.line_box.width > 0.0 {
                        //     line.finish_line_with_decorations(
                        //         nodes,
                        //         &mut inline_layout_ctx,
                        //         input.text,
                        //         ctx.float_ctx(),
                        //         None,
                        //     );
                        // }

                        let _ascent = layout_node.dimensions.height
                            + box_model.margin.top.to_px()
                            + box_model.margin.bottom.to_px();

                        layout_node.margin = box_model.margin;
                        // line.line_box.add(nodes, &mut layout_node, ascent, 0.0);

                        nodes[id.index()] = Some(layout_node);
                        inline_layout_ctx.ids.push(id);
                    }
                }
                InlineItem::Image(img) => {
                    layout_image(nodes, &mut inline_layout_ctx, input, img, ctx, &mut line);
                }
                InlineItem::Break { line_height_px } => {
                    line.finish_line_with_decorations(
                        nodes,
                        &mut inline_layout_ctx,
                        input.text,
                        ctx.float_ctx(),
                        Some(*line_height_px),
                    );
                }
            }
        }

        line.close_active_decorations(&mut inline_layout_ctx.inline_box_stack);

        let (_, line_height) = line.line_box.finish(
            nodes,
            ctx.float_ctx(),
            inline_ctx.containing_block.x,
            inline_ctx.containing_block.width,
            input.text.last_text_align,
            input.text.last_writing_mode,
        );

        let total_height = inline_layout_ctx.current_y + line_height - inline_ctx.containing_block.y;

        (
            inline_layout_ctx.ids,
            Rect::new(
                inline_ctx.containing_block.x,
                inline_ctx.containing_block.y,
                inline_ctx.containing_block.width,
                total_height,
            ),
        )
    }

    fn auto_inline_flow_root_width(
        nodes: &mut Vec<Option<LayoutNode>>,
        layout_node: &LayoutNode,
        padding: SideOffset,
        border: SideOffset,
    ) -> f64 {
        let content_left = layout_node.dimensions.x + padding.left + border.left;
        let max_right = layout_node
            .children
            .iter()
            .fold(content_left, |right, child| right.max(InlineLayout::max_right_edge(nodes, child)));
        let content_width = (max_right - content_left).max(0.0);

        content_width + padding.horizontal() + border.horizontal()
    }

    fn max_right_edge(nodes: &mut Vec<Option<LayoutNode>>, node_id: &LayoutNodeId) -> f64 {
        let Some(node) = std::mem::take(&mut nodes[node_id.index()]) else {
            return 0.0;
        };

        let mut right = node.dimensions.x + node.dimensions.width;
        for child in &node.children {
            right = right.max(InlineLayout::max_right_edge(nodes, child));
        }

        right
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_inline_flow_root_width_uses_descendant_extent() {
        let nested_text = LayoutNode::builder(LayoutNodeId::new(2))
            .dimensions(Rect::new(13.0, 0.0, 25.0, 12.0))
            .build();
        let inline_child = LayoutNode::builder(LayoutNodeId::new(1))
            .dimensions(Rect::new(13.0, 0.0, 25.0, 12.0))
            .children(vec![LayoutNodeId::new(2)])
            .build();
        let container = LayoutNode::builder(LayoutNodeId::new(0))
            .dimensions(Rect::new(8.0, 0.0, 500.0, 20.0))
            .children(vec![LayoutNodeId::new(1)])
            .build();

        let mut nodes = vec![
            Some(container.clone()),
            Some(inline_child),
            Some(nested_text),
        ];

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

        let width = InlineLayout::auto_inline_flow_root_width(&mut nodes, &container, padding, border);

        assert!((width - 35.0).abs() < 0.001);
    }
}
