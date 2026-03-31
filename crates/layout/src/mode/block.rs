use css_style::{ComputedDimension, ComputedStyle, Position, StyledNode};
use css_values::display::{Clear, Float, OutsideDisplay};

use crate::{
    LayoutColors, LayoutEngine, LayoutNode, Rect, SideOffset, TextContext, context::ImageContext, float::FloatContext,
    layout::LayoutContext, mode::inline::InlineLayout, position::PositionContext, resolver::PropertyResolver,
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
    pub fn new(style: &ComputedStyle) -> Self {
        Self {
            current_y: 0.0,
            previous_margin_bottom: 0.0,
            parent_has_top_fence: PropertyResolver::has_top_fence(style),
            is_first_child: true,
        }
    }

    fn advance(&mut self, child_margin_top: f32, child_height: f32, child_margin_bottom: f32, is_float: bool) -> f32 {
        if is_float {
            return self.current_y;
        }

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

    /// Advance the flow to account for a child positioned at `child_y_offset` with given height.
    /// This is used when clearance or other factors have already determined the child's y position.
    fn advance_to(&mut self, child_y_offset: f32, child_height: f32, child_margin_bottom: f32, is_float: bool) {
        if is_float {
            return;
        }

        self.is_first_child = false;
        self.current_y = child_y_offset + child_height;
        self.previous_margin_bottom = child_margin_bottom;
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
        styled_node: &StyledNode,
        ctx: &mut LayoutContext,
        position_ctx: &mut PositionContext,
        float_ctx: &mut FloatContext,
        text_ctx: &mut TextContext,
        image_ctx: &ImageContext,
    ) -> Option<LayoutNode> {
        if styled_node.style.position.is_out_of_flow() && !ctx.bypass {
            position_ctx.defer(styled_node.clone());

            return None;
        }

        let container_width = ctx.containing_block().width;

        let (margin, padding, border) = PropertyResolver::resolve_box_model(&styled_node.style);
        let box_width = PropertyResolver::calculate_width(styled_node, container_width);

        let content_width = if styled_node.style.width == ComputedDimension::Auto {
            (box_width - padding.horizontal() - border.horizontal()).max(0.0)
        } else {
            box_width
        };

        let x = Self::calculate_x(styled_node, ctx, &margin, &padding, &border, content_width);
        let y = ctx.containing_block().y + ctx.block_cursor.y;

        let mut children = Vec::with_capacity(styled_node.children.len());
        let mut flow = BlockFlow::new(&styled_node.style);
        let child_containing_height = match styled_node.style.height {
            ComputedDimension::Auto => 0.0,
            _ => PropertyResolver::calculate_height(styled_node, 0.0, ctx.containing_block().height).max(0.0),
        };

        if styled_node.style.position != Position::Static {
            position_ctx.push_position(Rect::new(
                x,
                y,
                box_width,
                child_containing_height + padding.vertical() + border.vertical(),
            ));
        }

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
                    &styled_node.style,
                    &styled_node.children[inline_start..inline_end],
                    image_ctx,
                );

                if !inline_items.is_empty() {
                    let inline_x = x + padding.left + border.left;
                    let inline_y = y + padding.top + border.top + flow.current_y;
                    let inline_width = content_width;

                    let (inline_layout_nodes, inline_height) = InlineLayout::layout(
                        &inline_items,
                        text_ctx,
                        float_ctx,
                        inline_width,
                        inline_x,
                        inline_y,
                        image_ctx,
                    );

                    if !inline_layout_nodes.is_empty() || inline_height > 0.0 {
                        children.extend(inline_layout_nodes);

                        flow.current_y += inline_height;
                        flow.previous_margin_bottom = 0.0;
                        flow.is_first_child = false;
                    }
                }

                continue;
            }

            let child_margin = PropertyResolver::resolve_margin(&child_style_node.style);

            let temp_clearance = if flow.is_first_child {
                if flow.parent_has_top_fence {
                    child_margin.top
                } else {
                    0.0
                }
            } else {
                BlockFlow::collapse_margins(flow.previous_margin_bottom, child_margin.top)
            };

            let mut child_y_offset = flow.current_y + temp_clearance;

            let clear = child_style_node.style.clear;
            let has_clearance = if clear != Clear::None {
                let absolute_y = y + padding.top + border.top + child_y_offset;
                let cleared_y = float_ctx.clear_y(clear, child_style_node.style.writing_mode, absolute_y);
                let relative_cleared_y = cleared_y - (y + padding.top + border.top);
                if relative_cleared_y > child_y_offset {
                    child_y_offset = relative_cleared_y;
                    true
                } else {
                    false
                }
            } else {
                false
            };

            let mut child_ctx = LayoutContext::new(Rect::new(
                x + padding.left + border.left,
                y + padding.top + border.top,
                content_width,
                child_containing_height,
            ));

            child_ctx.block_cursor.y = child_y_offset;

            if let Some(child_node) = LayoutEngine::layout_node(
                child_style_node,
                &mut child_ctx,
                position_ctx,
                float_ctx,
                text_ctx,
                image_ctx,
            ) {
                if has_clearance {
                    flow.advance_to(
                        child_y_offset,
                        child_node.dimensions.height,
                        child_node.margin.bottom,
                        child_style_node.style.float != Float::None,
                    );
                } else {
                    flow.advance(
                        child_node.margin.top,
                        child_node.dimensions.height,
                        child_node.margin.bottom,
                        child_style_node.style.float != Float::None,
                    );
                }

                if child_style_node.style.float != Float::None {
                    float_ctx.add_float(
                        Rect::new(
                            child_node.dimensions.x,
                            y + padding.top + border.top + child_y_offset,
                            child_node.dimensions.width,
                            child_node.dimensions.height,
                        ),
                        child_style_node.style.writing_mode,
                        child_style_node.style.float,
                    );
                }

                children.push(child_node);
            }

            child_idx += 1;
        }

        let has_bottom_fence = PropertyResolver::has_bottom_fence(&styled_node.style);

        let content_height_from_children = if !has_bottom_fence && !children.is_empty() {
            flow.current_y
        } else {
            flow.current_y + flow.previous_margin_bottom
        };

        let calculated_height = PropertyResolver::calculate_height(
            styled_node,
            content_height_from_children,
            ctx.containing_block().height,
        );

        let final_height = if styled_node.style.height == ComputedDimension::Auto {
            content_height_from_children + padding.vertical() + border.vertical()
        } else {
            calculated_height + padding.vertical() + border.vertical()
        };

        let mut margin = margin;
        if !flow.parent_has_top_fence && !children.is_empty() {
            margin.top = f32::max(margin.top, children[0].margin.top);
        }
        if !has_bottom_fence && !children.is_empty() {
            margin.bottom = f32::max(margin.bottom, children.last().unwrap().margin.bottom);
        }

        let colors = LayoutColors::from(styled_node);

        Some(
            LayoutNode::builder(styled_node.node_id)
                .margin(margin)
                .padding(padding)
                .border(border)
                .colors(colors)
                .cursor(styled_node.style.cursor)
                .children(children)
                .height_auto(styled_node.style.height == ComputedDimension::Auto)
                .dimensions(Rect::new(x, y, content_width + padding.horizontal() + border.horizontal(), final_height))
                .build(),
        )
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

        if styled_node.style.float == Float::Left {
            ctx.containing_block().x + margin.left
        } else if styled_node.style.float == Float::Right {
            ctx.containing_block().x + container_width - margin.right - total_width
        } else if styled_node.style.margin_left_auto && styled_node.style.margin_right_auto {
            ctx.containing_block().x + (container_width - total_width) / 2.0
        } else if styled_node.style.margin_left_auto {
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
        let mut flow = BlockFlow::new(&ComputedStyle::default());

        let y1 = flow.advance(10.0, 50.0, 15.0, false);
        assert_eq!(y1, 0.0);
        assert_eq!(flow.current_y, 50.0);

        let y2 = flow.advance(20.0, 30.0, 10.0, false);
        assert_eq!(y2, 70.0);
        assert_eq!(flow.current_y, 100.0);

        let y3 = flow.advance(5.0, 40.0, 20.0, false);
        assert_eq!(y3, 110.0);
        assert_eq!(flow.current_y, 150.0);
    }

    #[test]
    fn test_calculate_x() {
        let style = ComputedStyle {
            margin_left_auto: true,
            margin_right_auto: true,
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

        let x = BlockLayout::calculate_x(&styled_node, &ctx, &margin, &padding, &border, content_width);

        assert_eq!(x, 200.0);
    }
}
