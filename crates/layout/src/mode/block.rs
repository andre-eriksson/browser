use css_style::{ComputedDimension, ComputedStyle, Position, StyledNode};
use css_values::display::{Clear, Float, OutsideDisplay};

use crate::{
    LayoutColors, LayoutEngine, LayoutNode, Rect, SideOffset, TextContext,
    context::ImageContext,
    float::FloatContext,
    layout::LayoutContext,
    mode::inline::{InlineContext, InlineLayout},
    position::PositionContext,
    resolver::PropertyResolver,
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
        if styled_node.style.position.is_out_of_flow() && ctx.deferred {
            let containing_block = if styled_node.style.position == Position::Fixed {
                ctx.containing_block()
            } else {
                ctx.positioned_containing_block()
            };
            position_ctx.defer(styled_node.clone(), containing_block);

            return None;
        }

        let container_width = ctx.containing_block().width;

        let (margin, padding, border) = PropertyResolver::resolve_box_model(&styled_node.style);

        let width = Self::calculate_width(styled_node, container_width, &padding, &border);
        let x = Self::calculate_x(styled_node, ctx, &margin, &padding, &border, width);
        let y = Self::calculate_y(styled_node, ctx);

        let mut children = Vec::with_capacity(styled_node.children.len());
        let mut flow = BlockFlow::new(&styled_node.style);
        let child_containing_height = Self::calculate_height(styled_node, ctx.containing_block().height);

        let parent_positioned_cb = ctx.positioned_containing_block();

        if styled_node.style.position != Position::Static {
            position_ctx.push_position(Rect::new(
                x,
                y,
                width,
                child_containing_height + padding.vertical() + border.vertical(),
            ));
        }

        if styled_node.style.position == Position::Static {
            ctx.set_positioned_containing_block(parent_positioned_cb);
        } else {
            ctx.set_positioned_containing_block(Rect::new(
                x,
                y,
                width,
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
                    let inline_ctx = InlineContext {
                        start_x: x + padding.left + border.left,
                        start_y: y + padding.top + border.top + flow.current_y,
                        available_width: width,
                    };

                    let (inline_layout_nodes, inline_height) =
                        InlineLayout::layout(&inline_items, text_ctx, position_ctx, float_ctx, image_ctx, inline_ctx);

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
                width,
                child_containing_height,
            ));
            child_ctx.set_positioned_containing_block(ctx.positioned_containing_block());

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
            if child_containing_height > calculated_height {
                child_containing_height + padding.vertical() + border.vertical()
            } else {
                calculated_height + padding.vertical() + border.vertical()
            }
        };

        let mut margin = margin;
        if !flow.parent_has_top_fence && !children.is_empty() {
            margin.top = f32::max(margin.top, children[0].margin.top);
        }
        if !has_bottom_fence && !children.is_empty() {
            margin.bottom = f32::max(margin.bottom, children.last().unwrap().margin.bottom);
        }

        let colors = LayoutColors::from(styled_node);

        let node = LayoutNode::builder(styled_node.node_id)
            .margin(margin)
            .padding(padding)
            .border(border)
            .colors(colors)
            .cursor(styled_node.style.cursor)
            .children(children)
            .height_auto(styled_node.style.height == ComputedDimension::Auto)
            .position(styled_node.style.position)
            .dimensions(Rect::new(x, y, width + padding.horizontal() + border.horizontal(), final_height))
            .build();

        Some(node)
    }

    fn is_inline(node: &StyledNode) -> bool {
        node.style.display.outside() == Some(OutsideDisplay::Inline)
    }

    fn calculate_width(
        styled_node: &StyledNode,
        container_width: f32,
        padding: &SideOffset,
        border: &SideOffset,
    ) -> f32 {
        let style = &styled_node.style;
        if style.position.is_out_of_flow() {
            let has_left = !style.left_auto;
            let has_right = !style.right_auto;
            let width_is_auto = style.width == ComputedDimension::Auto;

            if has_left && has_right && width_is_auto {
                return container_width - style.left - style.right;
            }
        }

        let specified_width = PropertyResolver::calculate_width(styled_node, container_width);
        if styled_node.style.width == ComputedDimension::Auto {
            (specified_width - padding.horizontal() - border.horizontal()).max(0.0)
        } else {
            specified_width
        }
    }

    fn calculate_x(
        styled_node: &StyledNode,
        ctx: &LayoutContext,
        margin: &SideOffset,
        padding: &SideOffset,
        border: &SideOffset,
        content_width: f32,
    ) -> f32 {
        let style = &styled_node.style;
        let container_width = ctx.containing_block().width;
        let has_left = !style.left_auto;
        let has_right = !style.right_auto;

        let total_width = content_width + padding.horizontal() + border.horizontal();
        let normal_x = if styled_node.style.float == Float::Left {
            ctx.containing_block().x + margin.left
        } else if styled_node.style.float == Float::Right {
            ctx.containing_block().x + container_width - margin.right - total_width
        } else if styled_node.style.margin_left_auto && styled_node.style.margin_right_auto {
            ctx.containing_block().x + (container_width - total_width) / 2.0
        } else if styled_node.style.margin_left_auto {
            ctx.containing_block().x + container_width - margin.right - total_width
        } else {
            ctx.containing_block().x + margin.left
        };

        if style.position.is_out_of_flow() {
            if has_left {
                return ctx.containing_block().x + style.left;
            } else if has_right {
                return ctx.containing_block().x + container_width - style.right - total_width;
            }
        } else if style.position == Position::Relative {
            if has_left {
                return normal_x + style.left;
            } else if has_right {
                return normal_x - style.right;
            }
        }

        normal_x
    }

    fn calculate_y(styled_node: &StyledNode, ctx: &LayoutContext) -> f32 {
        let style = &styled_node.style;
        let has_top = !style.top_auto;
        let has_bottom = !style.bottom_auto;
        let normal_y = ctx.containing_block().y + ctx.block_cursor.y;

        if style.position.is_out_of_flow() && has_top {
            return ctx.containing_block().y + style.top + style.margin_top;
        } else if style.position == Position::Relative {
            if has_top {
                return normal_y + style.top;
            } else if has_bottom {
                return normal_y - style.bottom;
            }
        }

        normal_y
    }

    fn calculate_height(styled_node: &StyledNode, containing_block_height: f32) -> f32 {
        let style = &styled_node.style;
        let height_is_unconstrained =
            style.height == ComputedDimension::Auto || style.height == ComputedDimension::Percentage(100.0);
        let has_top = !style.top_auto;
        let has_bottom = !style.bottom_auto;

        if style.position.is_out_of_flow() && has_top && has_bottom && height_is_unconstrained {
            (containing_block_height - style.top - style.bottom).max(0.0)
        } else {
            match styled_node.style.height {
                ComputedDimension::Auto => 0.0,
                _ => PropertyResolver::calculate_height(styled_node, 0.0, containing_block_height).max(0.0),
            }
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
    fn test_calculate_x_static() {
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

        let margin = SideOffset::zero();
        let padding = SideOffset::zero();
        let border = SideOffset::zero();
        let content_width = 400.0;

        let x = BlockLayout::calculate_x(&styled_node, &ctx, &margin, &padding, &border, content_width);

        assert_eq!(x, 200.0);
    }

    #[test]
    fn test_calculate_x_float_left() {
        let style = ComputedStyle {
            float: Float::Left,
            ..Default::default()
        };

        let styled_node = StyledNode {
            style,
            ..StyledNode::new(NodeId(0))
        };

        let ctx = LayoutContext::new(viewport());

        let margin = SideOffset::zero();
        let padding = SideOffset::zero();
        let border = SideOffset::zero();
        let content_width = 200.0;

        let x = BlockLayout::calculate_x(&styled_node, &ctx, &margin, &padding, &border, content_width);

        assert_eq!(x, 0.0);
    }

    #[test]
    fn test_calculate_x_float_right() {
        let style = ComputedStyle {
            float: Float::Right,
            ..Default::default()
        };

        let styled_node = StyledNode {
            style,
            ..StyledNode::new(NodeId(0))
        };

        let ctx = LayoutContext::new(viewport());

        let margin = SideOffset::zero();
        let padding = SideOffset::zero();
        let border = SideOffset::zero();
        let content_width = 200.0;

        let x = BlockLayout::calculate_x(&styled_node, &ctx, &margin, &padding, &border, content_width);

        assert_eq!(x, 600.0);
    }

    #[test]
    fn test_calculate_x_absolute_left_precedence_over_right() {
        let style = ComputedStyle {
            position: Position::Absolute,
            left: 50.0,
            left_auto: false,
            right: 30.0,
            right_auto: false,
            ..Default::default()
        };

        let styled_node = StyledNode {
            style,
            ..StyledNode::new(NodeId(0))
        };

        let ctx = LayoutContext::new(viewport());
        let x = BlockLayout::calculate_x(
            &styled_node,
            &ctx,
            &SideOffset::zero(),
            &SideOffset::zero(),
            &SideOffset::zero(),
            200.0,
        );

        assert_eq!(x, 50.0);
    }

    #[test]
    fn test_calculate_x_fixed_right_when_left_auto() {
        let style = ComputedStyle {
            position: Position::Fixed,
            right: 30.0,
            right_auto: false,
            ..Default::default()
        };

        let styled_node = StyledNode {
            style,
            ..StyledNode::new(NodeId(0))
        };

        let ctx = LayoutContext::new(viewport());
        let x = BlockLayout::calculate_x(
            &styled_node,
            &ctx,
            &SideOffset::zero(),
            &SideOffset::zero(),
            &SideOffset::zero(),
            200.0,
        );

        assert_eq!(x, 570.0);
    }

    #[test]
    fn test_calculate_x_relative_left_offsets_from_normal_position() {
        let style = ComputedStyle {
            position: Position::Relative,
            left: 25.0,
            left_auto: false,
            ..Default::default()
        };

        let styled_node = StyledNode {
            style,
            ..StyledNode::new(NodeId(0))
        };

        let ctx = LayoutContext::new(viewport());
        let margin = SideOffset {
            left: 40.0,
            ..SideOffset::zero()
        };
        let x = BlockLayout::calculate_x(&styled_node, &ctx, &margin, &SideOffset::zero(), &SideOffset::zero(), 200.0);

        assert_eq!(x, 65.0);
    }

    #[test]
    fn test_calculate_x_relative_right_offsets_from_normal_position() {
        let style = ComputedStyle {
            position: Position::Relative,
            right: 30.0,
            right_auto: false,
            ..Default::default()
        };

        let styled_node = StyledNode {
            style,
            ..StyledNode::new(NodeId(0))
        };

        let ctx = LayoutContext::new(viewport());
        let margin = SideOffset {
            left: 40.0,
            ..SideOffset::zero()
        };
        let x = BlockLayout::calculate_x(&styled_node, &ctx, &margin, &SideOffset::zero(), &SideOffset::zero(), 200.0);

        assert_eq!(x, 10.0);
    }

    #[test]
    fn test_calculate_y_absolute_top_uses_containing_block() {
        let style = ComputedStyle {
            position: Position::Absolute,
            top: 20.0,
            top_auto: false,
            ..Default::default()
        };

        let styled_node = StyledNode {
            style,
            ..StyledNode::new(NodeId(0))
        };

        let mut ctx = LayoutContext::new(viewport());
        ctx.block_cursor.y = 120.0;

        let y = BlockLayout::calculate_y(&styled_node, &ctx);
        assert_eq!(y, 20.0);
    }

    #[test]
    fn test_calculate_y_absolute_top_includes_margin_top() {
        let style = ComputedStyle {
            position: Position::Absolute,
            top: 20.0,
            top_auto: false,
            margin_top: 12.0,
            ..Default::default()
        };

        let styled_node = StyledNode {
            style,
            ..StyledNode::new(NodeId(0))
        };

        let ctx = LayoutContext::new(viewport());
        let y = BlockLayout::calculate_y(&styled_node, &ctx);
        assert_eq!(y, 32.0);
    }

    #[test]
    fn test_calculate_y_relative_top_offsets_from_flow_position() {
        let style = ComputedStyle {
            position: Position::Relative,
            top: 15.0,
            top_auto: false,
            ..Default::default()
        };

        let styled_node = StyledNode {
            style,
            ..StyledNode::new(NodeId(0))
        };

        let mut ctx = LayoutContext::new(viewport());
        ctx.block_cursor.y = 120.0;

        let y = BlockLayout::calculate_y(&styled_node, &ctx);
        assert_eq!(y, 135.0);
    }

    #[test]
    fn test_calculate_y_relative_bottom_offsets_up_from_flow_position() {
        let style = ComputedStyle {
            position: Position::Relative,
            bottom: 18.0,
            bottom_auto: false,
            ..Default::default()
        };

        let styled_node = StyledNode {
            style,
            ..StyledNode::new(NodeId(0))
        };

        let mut ctx = LayoutContext::new(viewport());
        ctx.block_cursor.y = 120.0;

        let y = BlockLayout::calculate_y(&styled_node, &ctx);
        assert_eq!(y, 102.0);
    }

    #[test]
    fn test_calculate_height_absolute_auto_with_top_and_bottom() {
        let style = ComputedStyle {
            position: Position::Absolute,
            top: 20.0,
            top_auto: false,
            bottom: 30.0,
            bottom_auto: false,
            height: ComputedDimension::Auto,
            ..Default::default()
        };

        let styled_node = StyledNode {
            style,
            ..StyledNode::new(NodeId(0))
        };

        let height = BlockLayout::calculate_height(&styled_node, 600.0);
        assert_eq!(height, 550.0);
    }

    #[test]
    fn test_calculate_height_fixed_100_percent_with_top_and_bottom() {
        let style = ComputedStyle {
            position: Position::Fixed,
            top: 10.0,
            top_auto: false,
            bottom: 40.0,
            bottom_auto: false,
            height: ComputedDimension::Percentage(100.0),
            ..Default::default()
        };

        let styled_node = StyledNode {
            style,
            ..StyledNode::new(NodeId(0))
        };

        let height = BlockLayout::calculate_height(&styled_node, 600.0);
        assert_eq!(height, 550.0);
    }

    #[test]
    fn test_calculate_height_relative_auto_ignores_top_and_bottom() {
        let style = ComputedStyle {
            position: Position::Relative,
            top: 30.0,
            top_auto: false,
            bottom: 20.0,
            bottom_auto: false,
            height: ComputedDimension::Auto,
            ..Default::default()
        };

        let styled_node = StyledNode {
            style,
            ..StyledNode::new(NodeId(0))
        };

        let height = BlockLayout::calculate_height(&styled_node, 600.0);
        assert_eq!(height, 0.0);
    }
}
