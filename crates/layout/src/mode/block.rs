use css_style::{ComputedSize, ComputedStyle, Position, StyleTree};
use css_values::display::{Clear, Float};
use html_dom::{DocumentRoot, NodeId};

use crate::{
    LayoutColors, LayoutEngine, LayoutNode, Margin, Rect, TextContext, engine::LayoutMode, layout::LayoutContext,
    primitives::SideOffset, resolver::PropertyResolver,
};

#[derive(Debug, Clone, Default, Copy)]
pub struct BlockContext {
    pub y: f64,
}

impl From<f64> for BlockContext {
    fn from(y: f64) -> Self {
        Self { y }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BlockFlow {
    current_y: f64,
    previous_margin_bottom: f64,
    parent_has_top_fence: bool,
    is_first_child: bool,
}

impl BlockFlow {
    pub fn new(style: &ComputedStyle, containing_width: f64) -> Self {
        Self {
            current_y: 0.0,
            previous_margin_bottom: 0.0,
            parent_has_top_fence: PropertyResolver::has_top_fence(style, containing_width),
            is_first_child: true,
        }
    }

    fn advance(&mut self, child_margin_top: f64, child_height: f64, child_margin_bottom: f64, is_float: bool) -> f64 {
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
    fn advance_to(&mut self, child_y_offset: f64, child_height: f64, child_margin_bottom: f64, is_float: bool) {
        if is_float {
            return;
        }

        self.is_first_child = false;
        self.current_y = child_y_offset + child_height;
        self.previous_margin_bottom = child_margin_bottom;
    }

    fn collapse_margins(a: f64, b: f64) -> f64 {
        if a >= 0.0 && b >= 0.0 {
            f64::max(a, b)
        } else if a < 0.0 && b < 0.0 {
            f64::min(a, b)
        } else {
            a + b
        }
    }
}

pub struct BlockLayout;

impl BlockLayout {
    pub fn layout(
        node_id: &NodeId,
        dom_tree: &DocumentRoot,
        style_tree: &StyleTree,
        ctx: &mut LayoutContext,
        text_ctx: &mut TextContext,
    ) -> Option<(LayoutNode, Rect)> {
        let style = &style_tree[node_id];
        let node = &dom_tree[node_id];
        let node_children = &node.children;

        if style.position.is_out_of_flow() && !ctx.is_deferred() {
            let containing_block = if style.position == Position::Fixed {
                ctx.containing_block()
            } else {
                ctx.positioned_containing_block()
            };
            ctx.position_ctx().defer(node_id, containing_block);

            return None;
        }

        let container_width = ctx.containing_block().width;

        let (margin, padding, border) = PropertyResolver::resolve_box_model(style, container_width);

        let width = Self::calculate_width(style, container_width, &padding, &border);
        let x = Self::calculate_x(style, ctx, &margin, &padding, &border, width);
        let y = Self::calculate_y(style, ctx);

        let mut flow = BlockFlow::new(style, container_width);
        let height = Self::calculate_height(style, ctx.containing_block().height);
        let mut child_width = 0.0;
        let mut child_height = 0.0;

        let parent_positioned_cb = ctx.positioned_containing_block();

        if style.position == Position::Static {
            ctx.set_positioned_containing_block(parent_positioned_cb);
        } else {
            let rect = Rect::new(x, y, width, height + padding.vertical() + border.vertical());

            ctx.position_ctx().push_position(rect);
            ctx.set_positioned_containing_block(rect);
        }

        let mut children = Vec::with_capacity(node_children.len());
        let child_len = node_children.len();
        let mut child_idx = 0;

        while child_idx < child_len {
            let child_style_node_id = &node_children[child_idx];
            let child_style = &style_tree[child_style_node_id];

            if Self::is_inline(child_style) {
                let inline_items =
                    LayoutEngine::collect_children(ctx, dom_tree, style_tree, node_id, &mut child_idx, |node_id| {
                        let style = &style_tree[node_id];
                        Self::is_inline(style)
                    });

                let containing_block = Rect::new(
                    x + padding.left + border.left,
                    y + padding.top + border.top + flow.current_y,
                    width,
                    height - flow.current_y,
                );
                let (inline_layout_nodes, inline_result) = LayoutEngine::layout_nodes(
                    dom_tree,
                    style_tree,
                    &inline_items,
                    LayoutMode::Inline,
                    style,
                    containing_block,
                    ctx,
                    text_ctx,
                );

                if !inline_layout_nodes.is_empty() || inline_result.height > 0.0 {
                    children.extend(inline_layout_nodes);

                    flow.current_y += inline_result.height;
                    flow.previous_margin_bottom = 0.0;
                    flow.is_first_child = false;
                }

                continue;
            }

            let child_margin = PropertyResolver::resolve_margin(child_style, container_width);

            let temp_clearance = if flow.is_first_child {
                if flow.parent_has_top_fence {
                    child_margin.top.to_px()
                } else {
                    0.0
                }
            } else {
                BlockFlow::collapse_margins(flow.previous_margin_bottom, child_margin.top.to_px())
            };

            let mut child_y_offset = flow.current_y + temp_clearance;

            let clear = child_style.clear;
            let has_clearance = if clear == Clear::None {
                false
            } else {
                let absolute_y = y + padding.top + border.top + child_y_offset;
                let cleared_y = ctx
                    .float_ctx()
                    .clear_y(clear, child_style.writing_mode, absolute_y);
                let relative_cleared_y = cleared_y - (y + padding.top + border.top);
                if relative_cleared_y > child_y_offset {
                    child_y_offset = relative_cleared_y;
                    true
                } else {
                    false
                }
            };

            let mut child_ctx = ctx.child_context(
                Rect::new(x + padding.left + border.left, y + padding.top + border.top, width, height),
                style.position.is_out_of_flow(),
            );
            child_ctx.set_positioned_containing_block(child_ctx.positioned_containing_block());
            child_ctx.block_cursor.y = child_y_offset;

            if let Some(child_node) =
                LayoutEngine::layout_node(dom_tree, style_tree, child_style_node_id, &mut child_ctx, text_ctx)
            {
                if has_clearance {
                    flow.advance_to(
                        child_y_offset,
                        child_node.0.dimensions.height,
                        child_node.0.margin.bottom.to_px(),
                        child_style.float != Float::None,
                    );
                } else {
                    flow.advance(
                        child_node.0.margin.top.to_px(),
                        child_node.0.dimensions.height,
                        child_node.0.margin.bottom.to_px(),
                        child_style.float != Float::None,
                    );
                }

                if child_style.float != Float::None {
                    ctx.float_ctx().add_float(
                        Rect::new(
                            child_node.0.dimensions.x,
                            y + padding.top + border.top + child_y_offset,
                            child_node.0.dimensions.width,
                            child_node.0.dimensions.height,
                        ),
                        child_style.writing_mode,
                        child_style.float,
                    );
                }

                child_width = f64::max(child_width, child_node.0.dimensions.width + child_node.0.margin.horizontal());
                child_height += child_node.1.height;
                children.push(child_node.0);
            }

            child_idx += 1;
        }

        let has_bottom_fence = PropertyResolver::has_bottom_fence(style, container_width);

        let content_height_from_children = if !has_bottom_fence && !children.is_empty() {
            flow.current_y
        } else {
            flow.current_y + flow.previous_margin_bottom
        };

        let calculated_height =
            PropertyResolver::calculate_height(style, content_height_from_children, ctx.containing_block().height);

        let final_height = if style.position.is_out_of_flow() {
            height
        } else if style.height == ComputedSize::Auto {
            content_height_from_children + padding.vertical() + border.vertical()
        } else if height > calculated_height {
            height + padding.vertical() + border.vertical()
        } else {
            calculated_height + padding.vertical() + border.vertical()
        };

        let final_width = width + padding.horizontal() + border.horizontal();

        let mut margin = margin;
        if !flow.parent_has_top_fence && !children.is_empty() {
            margin.top = f64::max(margin.top.to_px(), children[0].margin.top.to_px()).into();
        }
        if !has_bottom_fence && !children.is_empty() {
            margin.bottom = f64::max(margin.bottom.to_px(), children.last().unwrap().margin.bottom.to_px()).into();
        }

        let colors = LayoutColors::from(style);

        let node = LayoutNode::builder(*node_id)
            .margin(margin)
            .padding(padding)
            .border(border)
            .colors(colors)
            .cursor(style.cursor)
            .children(children)
            .height_auto(style.height == ComputedSize::Auto)
            .position(style.position)
            .dimensions(Rect::new(x, y, final_width, final_height))
            .build();

        Some((node, Rect::new(x, y, final_width.max(child_width), final_height.max(child_height))))
    }

    fn is_inline(style: &ComputedStyle) -> bool {
        LayoutMode::new(style) == Some(LayoutMode::Inline)
    }

    fn calculate_width(style: &ComputedStyle, container_width: f64, padding: &SideOffset, border: &SideOffset) -> f64 {
        if style.position.is_out_of_flow() {
            let has_left = !style.left.is_auto() && style.left.to_px(container_width) > 0.0;
            let has_right = !style.right.is_auto() && style.right.to_px(container_width) > 0.0;
            let width_is_auto = style.width == ComputedSize::Auto;

            if has_left && has_right && width_is_auto {
                return container_width - style.left.to_px(container_width) - style.right.to_px(container_width);
            }
        }

        let specified_width = PropertyResolver::calculate_width(style, container_width);
        if style.width == ComputedSize::Auto {
            (specified_width - padding.horizontal() - border.horizontal()).max(0.0)
        } else {
            specified_width
        }
    }

    fn calculate_x(
        style: &ComputedStyle,
        ctx: &LayoutContext,
        margin: &Margin,
        padding: &SideOffset,
        border: &SideOffset,
        content_width: f64,
    ) -> f64 {
        let container_width = ctx.containing_block().width;
        let has_left = !style.left.is_auto() && style.left.to_px(container_width) > 0.0;
        let has_right = !style.right.is_auto() && style.right.to_px(container_width) > 0.0;
        let margin_left_px = margin.left.to_px();
        let margin_right_px = margin.right.to_px();
        let left_px = style.left.to_px(container_width);
        let right_px = style.right.to_px(container_width);

        let total_width = content_width + padding.horizontal() + border.horizontal();
        let normal_x = if style.float == Float::Left {
            ctx.containing_block().x + margin_left_px
        } else if style.float == Float::Right {
            ctx.containing_block().x + container_width - margin_right_px - total_width
        } else if style.margin_left.is_auto() && style.margin_right.is_auto() {
            ctx.containing_block().x + (container_width - total_width) / 2.0
        } else if style.margin_left.is_auto() {
            ctx.containing_block().x + container_width - margin_right_px - total_width
        } else {
            ctx.containing_block().x + margin_left_px
        };

        if style.position.is_out_of_flow() {
            if has_left {
                return ctx.containing_block().x + left_px;
            } else if has_right {
                return ctx.containing_block().x + container_width - right_px - total_width;
            }
        } else if style.position == Position::Relative {
            if has_left {
                return normal_x + left_px;
            } else if has_right {
                return normal_x - right_px;
            }
        }

        normal_x
    }

    fn calculate_y(style: &ComputedStyle, ctx: &LayoutContext) -> f64 {
        let has_top = !style.top.is_auto() && style.top.to_px(ctx.containing_block().width) > 0.0;
        let has_bottom = !style.bottom.is_auto() && style.bottom.to_px(ctx.containing_block().width) > 0.0;
        let normal_y = ctx.containing_block().y + ctx.block_cursor.y;

        if style.position.is_out_of_flow() && has_top {
            return ctx.containing_block().y
                + style.top.to_px(ctx.containing_block().width)
                + style.margin_top.to_px(ctx.containing_block().width);
        } else if style.position == Position::Relative {
            if has_top {
                return normal_y + style.top.to_px(ctx.containing_block().width);
            } else if has_bottom {
                return normal_y - style.bottom.to_px(ctx.containing_block().width);
            }
        }

        normal_y
    }

    fn calculate_height(style: &ComputedStyle, containing_block_height: f64) -> f64 {
        let height_is_unconstrained =
            style.height == ComputedSize::Auto || style.height == ComputedSize::Percentage(100.0);
        let has_top = !style.top.is_auto() && style.top.to_px(containing_block_height) > 0.0;
        let has_bottom = !style.bottom.is_auto() && style.bottom.to_px(containing_block_height) > 0.0;

        if style.position.is_out_of_flow() && has_top && has_bottom && height_is_unconstrained {
            let top_px = style.top.to_px(containing_block_height);
            let bottom_px = style.bottom.to_px(containing_block_height);

            (containing_block_height - top_px - bottom_px).max(0.0)
        } else {
            match style.height {
                ComputedSize::Auto => 0.0,
                _ => PropertyResolver::calculate_height(style, 0.0, containing_block_height).max(0.0),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ImageContext, position::PositionContext};
    use css_style::{ComputedMargin, ComputedStyle};

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
        let mut flow = BlockFlow::new(&ComputedStyle::default(), viewport().width);

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
            margin_left: ComputedMargin::Auto,
            margin_right: ComputedMargin::Auto,
            ..Default::default()
        };

        let img_ctx = ImageContext::new();
        let mut position_ctx = PositionContext::new(viewport());
        let ctx = LayoutContext::new(viewport(), &img_ctx, &mut position_ctx);

        let margin = Margin::zero();
        let padding = SideOffset::zero();
        let border = SideOffset::zero();
        let content_width = 400.0;

        let x = BlockLayout::calculate_x(&style, &ctx, &margin, &padding, &border, content_width);

        assert_eq!(x, 200.0);
    }

    #[test]
    fn test_calculate_x_float_left() {
        let style = ComputedStyle {
            float: Float::Left,
            ..Default::default()
        };

        let img_ctx = ImageContext::new();
        let mut position_ctx = PositionContext::new(viewport());
        let ctx = LayoutContext::new(viewport(), &img_ctx, &mut position_ctx);

        let margin = Margin::zero();
        let padding = SideOffset::zero();
        let border = SideOffset::zero();
        let content_width = 200.0;

        let x = BlockLayout::calculate_x(&style, &ctx, &margin, &padding, &border, content_width);

        assert_eq!(x, 0.0);
    }

    #[test]
    fn test_calculate_x_float_right() {
        let style = ComputedStyle {
            float: Float::Right,
            ..Default::default()
        };

        let img_ctx = ImageContext::new();
        let mut position_ctx = PositionContext::new(viewport());
        let ctx = LayoutContext::new(viewport(), &img_ctx, &mut position_ctx);

        let margin = Margin::zero();
        let padding = SideOffset::zero();
        let border = SideOffset::zero();
        let content_width = 200.0;

        let x = BlockLayout::calculate_x(&style, &ctx, &margin, &padding, &border, content_width);

        assert_eq!(x, 600.0);
    }

    #[test]
    fn test_calculate_x_absolute_left_precedence_over_right() {
        let style = ComputedStyle {
            position: Position::Absolute,
            left: 50.0.into(),
            right: 30.0.into(),
            ..Default::default()
        };

        let img_ctx = ImageContext::new();
        let mut position_ctx = PositionContext::new(viewport());
        let ctx = LayoutContext::new(viewport(), &img_ctx, &mut position_ctx);

        let x =
            BlockLayout::calculate_x(&style, &ctx, &Margin::zero(), &SideOffset::zero(), &SideOffset::zero(), 200.0);

        assert_eq!(x, 50.0);
    }

    #[test]
    fn test_calculate_x_fixed_right_when_left_auto() {
        let style = ComputedStyle {
            position: Position::Fixed,
            right: 30.0.into(),
            left: ComputedMargin::Auto,
            ..Default::default()
        };

        let img_ctx = ImageContext::new();
        let mut position_ctx = PositionContext::new(viewport());
        let ctx = LayoutContext::new(viewport(), &img_ctx, &mut position_ctx);

        let x =
            BlockLayout::calculate_x(&style, &ctx, &Margin::zero(), &SideOffset::zero(), &SideOffset::zero(), 200.0);

        assert_eq!(x, 570.0);
    }

    #[test]
    fn test_calculate_x_relative_left_offsets_from_normal_position() {
        let style = ComputedStyle {
            position: Position::Relative,
            left: 25.0.into(),
            ..Default::default()
        };

        let img_ctx = ImageContext::new();
        let mut position_ctx = PositionContext::new(viewport());
        let ctx = LayoutContext::new(viewport(), &img_ctx, &mut position_ctx);

        let margin = Margin {
            left: 40.0.into(),
            ..Margin::zero()
        };
        let x = BlockLayout::calculate_x(&style, &ctx, &margin, &SideOffset::zero(), &SideOffset::zero(), 200.0);

        assert_eq!(x, 65.0);
    }

    #[test]
    fn test_calculate_x_relative_right_offsets_from_normal_position() {
        let style = ComputedStyle {
            position: Position::Relative,
            right: 30.0.into(),
            ..Default::default()
        };

        let img_ctx = ImageContext::new();
        let mut position_ctx = PositionContext::new(viewport());
        let ctx = LayoutContext::new(viewport(), &img_ctx, &mut position_ctx);

        let margin = Margin {
            left: 40.0.into(),
            ..Margin::zero()
        };
        let x = BlockLayout::calculate_x(&style, &ctx, &margin, &SideOffset::zero(), &SideOffset::zero(), 200.0);

        assert_eq!(x, 10.0);
    }

    #[test]
    fn test_calculate_y_absolute_top_uses_containing_block() {
        let style = ComputedStyle {
            position: Position::Absolute,
            top: 20.0.into(),
            ..Default::default()
        };

        let img_ctx = ImageContext::new();
        let mut position_ctx = PositionContext::new(viewport());
        let mut ctx = LayoutContext::new(viewport(), &img_ctx, &mut position_ctx);

        ctx.block_cursor.y = 120.0;

        let y = BlockLayout::calculate_y(&style, &ctx);
        assert_eq!(y, 20.0);
    }

    #[test]
    fn test_calculate_y_absolute_top_includes_margin_top() {
        let style = ComputedStyle {
            position: Position::Absolute,
            top: 20.0.into(),
            margin_top: 12.0.into(),
            ..Default::default()
        };

        let img_ctx = ImageContext::new();
        let mut position_ctx = PositionContext::new(viewport());
        let ctx = LayoutContext::new(viewport(), &img_ctx, &mut position_ctx);

        let y = BlockLayout::calculate_y(&style, &ctx);
        assert_eq!(y, 32.0);
    }

    #[test]
    fn test_calculate_y_relative_top_offsets_from_flow_position() {
        let style = ComputedStyle {
            position: Position::Relative,
            top: 15.0.into(),
            ..Default::default()
        };

        let img_ctx = ImageContext::new();
        let mut position_ctx = PositionContext::new(viewport());
        let mut ctx = LayoutContext::new(viewport(), &img_ctx, &mut position_ctx);

        ctx.block_cursor.y = 120.0;

        let y = BlockLayout::calculate_y(&style, &ctx);
        assert_eq!(y, 135.0);
    }

    #[test]
    fn test_calculate_y_relative_bottom_offsets_up_from_flow_position() {
        let style = ComputedStyle {
            position: Position::Relative,
            bottom: 18.0.into(),
            ..Default::default()
        };

        let img_ctx = ImageContext::new();
        let mut position_ctx = PositionContext::new(viewport());
        let mut ctx = LayoutContext::new(viewport(), &img_ctx, &mut position_ctx);

        ctx.block_cursor.y = 120.0;

        let y = BlockLayout::calculate_y(&style, &ctx);
        assert_eq!(y, 102.0);
    }

    #[test]
    fn test_calculate_height_absolute_auto_with_top_and_bottom() {
        let style = ComputedStyle {
            position: Position::Absolute,
            top: 20.0.into(),
            bottom: 30.0.into(),
            height: ComputedSize::Auto,
            ..Default::default()
        };

        let height = BlockLayout::calculate_height(&style, 600.0);
        assert_eq!(height, 550.0);
    }

    #[test]
    fn test_calculate_height_fixed_100_percent_with_top_and_bottom() {
        let style = ComputedStyle {
            position: Position::Fixed,
            top: 10.0.into(),
            bottom: 40.0.into(),
            height: ComputedSize::Percentage(100.0),
            ..Default::default()
        };

        let height = BlockLayout::calculate_height(&style, 600.0);
        assert_eq!(height, 550.0);
    }

    #[test]
    fn test_calculate_height_relative_auto_ignores_top_and_bottom() {
        let style = ComputedStyle {
            position: Position::Relative,
            top: 30.0.into(),
            bottom: 20.0.into(),
            height: ComputedSize::Auto,
            ..Default::default()
        };

        let height = BlockLayout::calculate_height(&style, 600.0);
        assert_eq!(height, 0.0);
    }
}
