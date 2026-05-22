use css_display::BoxNode;
use css_style::{ComputedSize, ComputedStyle, Position, StyleTree};
use css_values::display::{Clear, Float, OutsideDisplay};

use crate::{
    LayoutColors, LayoutNode, Margin, Rect,
    context::{Geometry, LayoutContext},
    engine::LayoutInput,
    primitives::{SideOffset, Size},
};

pub struct BlockContext {
    pub cursor_y: f64,
    pub containing_width: f64,
    pub collapsed_margin: Option<f64>,
}
pub struct BlockLayout;

impl BlockLayout {
    pub fn layout(
        box_node: &BoxNode,
        parent_style: &ComputedStyle,
        input: &mut LayoutInput<'_>,
        ctx: &mut LayoutContext,
        block: &mut BlockContext,
    ) -> Option<(LayoutNode, Size)> {
        let style = &*box_node.style;

        // if style.position.is_out_of_flow() && !ctx.is_deferred() {
        //     let containing_block = if style.position == Position::Fixed {
        //         ctx.containing_block()
        //     } else {
        //         ctx.positioned_containing_block()
        //     };
        //     ctx.position_ctx().defer(&box_node, containing_block);

        //     return None;
        // }

        let container_width = ctx.containing_block().width;

        let (margin, padding, border) = Geometry::resolve_box_model(style, container_width);

        let (height, is_height_auto) =
            (Self::calculate_height(style, ctx.containing_block().height), style.height.is_auto());
        let width = Self::calculate_width(style, container_width, &padding, &border);
        let x = Self::calculate_x(style, ctx, &margin, &padding, &border, width);

        //let intrinsic_size =
        //    PropertyResolver::calculate_intrinsic_size(style, dom_tree, style_tree, node_children, ctx, text_ctx);

        // let flow = BlockFlow::new(style, container_width);
        // let parent_positioned_cb = ctx.positioned_containing_block();

        // if style.position == Position::Static {
        //     ctx.set_positioned_containing_block(parent_positioned_cb);
        // } else {
        //     let rect = Rect::new(x, y, width, height + padding.vertical() + border.vertical());

        //     ctx.position_ctx().push_position(rect);
        //     ctx.set_positioned_containing_block(rect);
        // }

        let containing_block = Rect::new(x + padding.left + border.left, 0.0, width, height);
        let mut block_ctx = BlockContext {
            cursor_y: 0.0,
            containing_width: width,
            collapsed_margin: None,
        };

        // for child in &box_node.children {}
        for child in box_node.children.iter() {
            BlockLayout::layout(child, style, input, ctx, &mut block_ctx);
        }

        if !style.margin_top.is_auto() && style.margin_top.to_px(width) > 0.0 {
            if let Some(collapsed) = block.collapsed_margin {
                block.collapsed_margin = Some(Self::collapse_margins(collapsed, style.margin_top.to_px(width)));
            } else {
                if let Some(collapsed) = block_ctx.collapsed_margin {
                    block.collapsed_margin = Some(Self::collapse_margins(collapsed, style.margin_top.to_px(width)));
                } else {
                    block.collapsed_margin = Some(style.margin_top.to_px(width));
                }
            }
        }

        // let final_height = if is_height_auto {
        //     child_size.height + padding.vertical() + border.vertical()
        // } else {
        //     height + padding.vertical() + border.vertical()
        // };

        // if !margin.bottom.is_auto() {
        //     block.collapsed_margin = Some(margin.bottom.to_px());
        // }

        if let Some(node_id) = box_node.node_id {
            let dom_node = &input.dom[node_id];

            if let Some(element) = dom_node.data.as_element() {
                eprintln!("--- {:?}, margin-top: {:?}", element, block.collapsed_margin);
            }
        }

        let colors = LayoutColors::from(style);

        let node = LayoutNode::builder(box_node.node_id)
            .border(border)
            .colors(colors)
            .cursor(style.cursor)
            .children(vec![])
            .height_auto(style.height == ComputedSize::Auto)
            .position(style.position)
            .dimensions(Rect::new(x, 0.0, width, 100.0))
            .build();

        Some((node, Size::new(width, 100.0)))
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

        let specified_width = Geometry::calculate_width(style, container_width);
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

    fn calculate_y(style: &ComputedStyle, ctx: &LayoutContext, block: &BlockContext, margin: &Margin) -> f64 {
        let has_top = !style.top.is_auto() && style.top.to_px(ctx.containing_block().width) > 0.0;
        let has_bottom = !style.bottom.is_auto() && style.bottom.to_px(ctx.containing_block().width) > 0.0;
        let normal_y = ctx.containing_block().y + block.cursor_y;

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
                _ => Geometry::calculate_height(style, 0.0, containing_block_height).max(0.0),
            }
        }
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

#[cfg(test)]
mod tests {
    use crate::context::{ImageContext, PositionContext};
    use css_style::{ComputedMargin, ComputedStyle};

    use super::*;

    fn viewport() -> Rect {
        Rect::new(0.0, 0.0, 800.0, 600.0)
    }

    // #[test]
    // fn test_collapsing_margins() {
    //     assert_eq!(BlockFlow::collapse_margins(10.0, 20.0), 20.0);
    //     assert_eq!(BlockFlow::collapse_margins(-10.0, -20.0), -20.0);
    //     assert_eq!(BlockFlow::collapse_margins(10.0, -5.0), 5.0);
    //     assert_eq!(BlockFlow::collapse_margins(-10.0, 5.0), -5.0);
    //     assert_eq!(BlockFlow::collapse_margins(0.0, 15.0), 15.0);
    //     assert_eq!(BlockFlow::collapse_margins(-5.0, 0.0), -5.0);
    // }

    // #[test]
    // fn test_advance_flow() {
    //     let mut flow = BlockFlow::new(&ComputedStyle::default(), viewport().width);

    //     let y1 = flow.advance(10.0, 50.0, 15.0, false);
    //     assert_eq!(y1, 0.0);
    //     assert_eq!(flow.current_y, 50.0);

    //     let y2 = flow.advance(20.0, 30.0, 10.0, false);
    //     assert_eq!(y2, 70.0);
    //     assert_eq!(flow.current_y, 100.0);

    //     let y3 = flow.advance(5.0, 40.0, 20.0, false);
    //     assert_eq!(y3, 110.0);
    //     assert_eq!(flow.current_y, 150.0);
    // }

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
        let ctx = LayoutContext::new(viewport(), &img_ctx, &mut position_ctx);
        let block = BlockContext {
            cursor_y: 120.0,
            containing_width: viewport().width,
            collapsed_margin: None,
        };
        let margin = Margin::zero();

        let y = BlockLayout::calculate_y(&style, &ctx, &block, &margin);
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
        let block = BlockContext {
            cursor_y: 0.0,
            containing_width: viewport().width,
            collapsed_margin: None,
        };
        let margin = Margin {
            top: 12.0.into(),
            ..Margin::zero()
        };

        let y = BlockLayout::calculate_y(&style, &ctx, &block, &margin);
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

        let block = BlockContext {
            cursor_y: 120.0,
            containing_width: viewport().width,
            collapsed_margin: None,
        };
        let margin = Margin::zero();

        let y = BlockLayout::calculate_y(&style, &ctx, &block, &margin);
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
        let block = BlockContext {
            cursor_y: 120.0,
            containing_width: viewport().width,
            collapsed_margin: None,
        };
        let margin = Margin::zero();

        let y = BlockLayout::calculate_y(&style, &ctx, &block, &margin);
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
