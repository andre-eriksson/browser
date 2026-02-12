use css_style::{AbsoluteContext, Dimension, OffsetValue, StyledNode, display::OutsideDisplay};

use crate::{
    LayoutColors, LayoutEngine, LayoutNode, Rect, SideOffset, TextContext, layout::LayoutContext,
    mode::inline::InlineLayout, resolver::PropertyResolver,
};

#[derive(Debug, Clone, Default, Copy)]
pub struct BlockCursor {
    pub y: f32,
}

impl From<f32> for BlockCursor {
    fn from(y: f32) -> Self {
        Self { y }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BlockFlow {
    current_y: f32,
    previous_margin_bottom: f32,
    parent_has_top_fence: bool,
    is_first_child: bool,
}

impl BlockFlow {
    pub fn new(padding_top: f32, border_top: f32) -> Self {
        Self {
            current_y: 0.0,
            previous_margin_bottom: 0.0,
            parent_has_top_fence: padding_top > 0.0 || border_top > 0.0,
            is_first_child: true,
        }
    }

    fn advance(
        &mut self,
        child_margin_top: f32,
        child_height: f32,
        child_margin_bottom: f32,
    ) -> f32 {
        let clearance;

        if self.is_first_child {
            if self.parent_has_top_fence {
                clearance = child_margin_top;
            } else {
                clearance = 0.0;
            }
            self.is_first_child = false;
        } else {
            clearance = Self::collapse_margins(self.previous_margin_bottom, child_margin_top);
        }

        let child_y = self.current_y + clearance;

        self.current_y = child_y + child_height;

        self.previous_margin_bottom = child_margin_bottom;

        child_y
    }

    fn collapse_margins(a: f32, b: f32) -> f32 {
        if a >= 0.0 && b >= 0.0 {
            f32::max(a, b)
        } else if a < 0.0 && b < 0.0 {
            f32::min(a, b)
        } else {
            a + b
        }
    }
}

pub struct BlockLayout;

impl BlockLayout {
    pub fn layout(
        absolute_ctx: &AbsoluteContext,
        styled_node: &StyledNode,
        ctx: &mut LayoutContext,
        text_ctx: &mut TextContext,
    ) -> LayoutNode {
        let font_size = styled_node.style.font_size;
        let container_width = ctx.containing_block().width;

        let (margin, padding, border) = PropertyResolver::resolve_box_model(
            absolute_ctx,
            &styled_node.style,
            container_width,
            font_size,
        );
        let box_width =
            PropertyResolver::calculate_width(absolute_ctx, styled_node, container_width);

        let content_width = if styled_node.style.width == Dimension::Auto {
            (box_width - padding.horizontal() - border.horizontal()).max(0.0)
        } else {
            box_width
        };

        let x = Self::calculate_x(styled_node, ctx, &margin, &padding, &border, content_width);
        let y = ctx.containing_block().y + ctx.block_cursor.y;

        let mut children = Vec::new();
        let mut flow = BlockFlow::new(padding.top, border.top);
        let child_containing_height = match styled_node.style.height {
            Dimension::Auto => 0.0,
            _ => PropertyResolver::calculate_height(
                absolute_ctx,
                styled_node,
                ctx.containing_block().height,
                0.0,
            )
            .max(0.0),
        };

        let child_len = styled_node.children.len();
        let mut child_idx = 0;

        while child_idx < child_len {
            let child_style_node = &styled_node.children[child_idx];

            if Self::is_inline(child_style_node) {
                let inline_start = child_idx;
                while child_idx < child_len && Self::is_inline(&styled_node.children[child_idx]) {
                    child_idx += 1;
                }
                let inline_end = child_idx;

                let inline_items = InlineLayout::collect_inline_items_from_nodes(
                    absolute_ctx,
                    &styled_node.style,
                    &styled_node.children[inline_start..inline_end],
                );

                if !inline_items.is_empty() {
                    let inline_x = x + padding.left + border.left;
                    let inline_y = y + padding.top + border.top + flow.current_y;
                    let inline_width = content_width;

                    let (inline_layout_nodes, inline_height) = InlineLayout::layout(
                        absolute_ctx,
                        &inline_items,
                        text_ctx,
                        inline_width,
                        inline_x,
                        inline_y,
                    );

                    if !inline_layout_nodes.is_empty() || inline_height > 0.0 {
                        for inline_node in inline_layout_nodes {
                            children.push(inline_node);
                        }

                        flow.current_y += inline_height;
                        flow.previous_margin_bottom = 0.0;
                        flow.is_first_child = false;
                    }
                }

                continue;
            }

            let child_margin = PropertyResolver::resolve_node_margins(
                absolute_ctx,
                child_style_node,
                content_width,
                font_size,
            );

            let temp_clearance = if flow.is_first_child {
                if flow.parent_has_top_fence {
                    child_margin.top
                } else {
                    0.0
                }
            } else {
                BlockFlow::collapse_margins(flow.previous_margin_bottom, child_margin.top)
            };

            let child_y_offset = flow.current_y + temp_clearance;

            let mut child_ctx = LayoutContext::new(Rect::new(
                x + padding.left + border.left,
                y + padding.top + border.top,
                content_width,
                child_containing_height,
            ));

            child_ctx.block_cursor.y = child_y_offset;

            let child_node =
                LayoutEngine::layout_node(absolute_ctx, child_style_node, &mut child_ctx, text_ctx);

            if let Some(child_node) = child_node {
                flow.advance(
                    child_node.resolved_margin.top,
                    child_node.dimensions.height,
                    child_node.resolved_margin.bottom,
                );
                children.push(child_node);
            }

            child_idx += 1;
        }

        let has_bottom_fence = padding.bottom > 0.0 || border.bottom > 0.0;

        let content_height_from_children = if !has_bottom_fence && !children.is_empty() {
            flow.current_y
        } else {
            flow.current_y + flow.previous_margin_bottom
        };

        let calculated_height = PropertyResolver::calculate_height(
            absolute_ctx,
            styled_node,
            ctx.containing_block().height,
            content_height_from_children,
        );

        let final_height = if styled_node.style.height == Dimension::Auto {
            content_height_from_children + padding.vertical() + border.vertical()
        } else {
            calculated_height + padding.vertical() + border.vertical()
        };

        let mut margin = margin;
        if !flow.parent_has_top_fence && !children.is_empty() {
            margin.top = f32::max(margin.top, children[0].resolved_margin.top);
        }
        if !has_bottom_fence && !children.is_empty() {
            margin.bottom = f32::max(
                margin.bottom,
                children.last().unwrap().resolved_margin.bottom,
            );
        }

        let colors = LayoutColors::from(styled_node);

        LayoutNode {
            node_id: styled_node.node_id,
            dimensions: Rect::new(
                x,
                y,
                content_width + padding.horizontal() + border.horizontal(),
                final_height,
            ),
            resolved_margin: margin,
            resolved_padding: padding,
            resolved_border: border,
            colors,
            children,
            text_buffer: None,
        }
    }

    fn is_inline(node: &StyledNode) -> bool {
        node.style.display.outside() == Some(OutsideDisplay::Inline)
    }

    fn calculate_x(
        styled_node: &StyledNode,
        ctx: &LayoutContext,
        margin: &SideOffset,
        padding: &SideOffset,
        border: &SideOffset,
        content_width: f32,
    ) -> f32 {
        let container_width = ctx.containing_block().width;
        let total_width = content_width + padding.horizontal() + border.horizontal();

        if styled_node.style.margin_left == OffsetValue::Auto
            && styled_node.style.margin_right == OffsetValue::Auto
        {
            ctx.containing_block().x + (container_width - total_width) / 2.0
        } else if styled_node.style.margin_left == OffsetValue::Auto {
            ctx.containing_block().x + container_width - margin.right - total_width
        } else {
            ctx.containing_block().x + margin.left
        }
    }
}

#[cfg(test)]
mod tests {
    use css_style::ComputedStyle;
    use html_dom::NodeId;

    use super::*;

    fn viewport() -> Rect {
        Rect::new(0.0, 0.0, 800.0, 600.0)
    }

    #[test]
    fn test_collapsing_margins() {
        assert_eq!(BlockFlow::collapse_margins(10.0, 20.0), 20.0);
        assert_eq!(BlockFlow::collapse_margins(-10.0, -20.0), -20.0);
        assert_eq!(BlockFlow::collapse_margins(10.0, -5.0), 5.0);
        assert_eq!(BlockFlow::collapse_margins(-10.0, 5.0), -5.0);
        assert_eq!(BlockFlow::collapse_margins(0.0, 15.0), 15.0);
        assert_eq!(BlockFlow::collapse_margins(-5.0, 0.0), -5.0);
    }

    #[test]
    fn test_advance_flow() {
        let mut flow = BlockFlow::new(0.0, 0.0);

        let y1 = flow.advance(10.0, 50.0, 15.0);
        assert_eq!(y1, 0.0);
        assert_eq!(flow.current_y, 50.0);

        let y2 = flow.advance(20.0, 30.0, 10.0);
        assert_eq!(y2, 70.0);
        assert_eq!(flow.current_y, 100.0);

        let y3 = flow.advance(5.0, 40.0, 20.0);
        assert_eq!(y3, 110.0);
        assert_eq!(flow.current_y, 150.0);
    }

    #[test]
    fn test_calculate_x() {
        let style = ComputedStyle {
            margin_left: OffsetValue::Auto,
            margin_right: OffsetValue::Auto,
            ..Default::default()
        };

        let styled_node = StyledNode {
            style,
            ..StyledNode::new(NodeId(0))
        };

        let ctx = LayoutContext::new(viewport());

        let margin = SideOffset {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        };
        let padding = SideOffset {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        };
        let border = SideOffset {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        };
        let content_width = 400.0;

        let x = BlockLayout::calculate_x(
            &styled_node,
            &ctx,
            &margin,
            &padding,
            &border,
            content_width,
        );

        assert_eq!(x, 200.0);
    }
}
