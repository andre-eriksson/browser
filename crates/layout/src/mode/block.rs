use css_style::{OffsetValue, Property, StyledNode, display::OutsideDisplay};
use html_dom::{HtmlTag, Tag};

use crate::{
    Color4f, LayoutColors, LayoutEngine, LayoutNode, Rect, SideOffset, TextContext,
    layout::LayoutContext, mode::inline::InlineLayout, resolver::PropertyResolver,
};

pub struct BlockCursor {
    pub y: f32,
}

pub struct BlockLayout;

impl BlockLayout {
    pub fn layout(
        styled_node: &StyledNode,
        ctx: &LayoutContext,
        cursor: &mut BlockCursor,
        text_ctx: &mut TextContext,
    ) -> LayoutNode {
        let font_size_px = styled_node.style.computed_font_size_px;

        let content_width =
            PropertyResolver::calculate_width(styled_node, ctx.containing_block.width);

        let margin = PropertyResolver::resolve_node_margins(
            styled_node,
            ctx.containing_block.width,
            font_size_px,
        );
        let padding = PropertyResolver::resolve_node_padding(
            styled_node,
            ctx.containing_block.width,
            font_size_px,
        );
        let border = PropertyResolver::resolve_node_borders(styled_node, font_size_px);

        let colors = LayoutColors {
            background_color: Color4f::from_css_color(&styled_node.style.background_color),
            color: Color4f::from_css_color(&styled_node.style.color),
        };

        let mut x = ctx.containing_block.x + margin.left;

        if let Ok(node_margin) = Property::resolve(&styled_node.style.margin) {
            if node_margin.left == OffsetValue::Auto && node_margin.right == OffsetValue::Auto {
                x = (ctx.containing_block.width - content_width) / 2.0;
            } else if node_margin.left == OffsetValue::Auto {
                x = ctx.containing_block.x + ctx.containing_block.width
                    - margin.right
                    - content_width;
            }
        }

        let y = Self::calculate_y_pos(styled_node, ctx, cursor, margin.top, padding.top);

        let mut collapsed_margin_top = 0.0;
        let mut collapsed_margin_bottom = 0.0;

        let mut children = Vec::new();

        let mut content_height = 0.0;
        let mut child_cursor = BlockCursor { y: 0.0 };

        let mut child_index = 0;
        let child_len = styled_node.children.len();
        let is_body = styled_node.tag == Some(Tag::Html(HtmlTag::Body));

        while child_index < child_len {
            let style_node = &styled_node.children[child_index];
            let display = Property::resolve(&style_node.style.display);

            if let Ok(display) = display
                && display.outside() == Some(OutsideDisplay::Inline)
            {
                let mut inline_end = child_index + 1;
                while inline_end < child_len
                    && let Ok(child_display) =
                        Property::resolve(&styled_node.children[inline_end].style.display)
                    && child_display.outside() == Some(OutsideDisplay::Inline)
                {
                    inline_end += 1;
                }

                let items = InlineLayout::collect_inline_items_from_nodes(
                    &styled_node.style,
                    &styled_node.children[child_index..inline_end],
                );

                let inline_y = y + child_cursor.y + padding.top + border.top;
                let (inline_nodes, inline_height) = InlineLayout::layout(
                    &items,
                    text_ctx,
                    content_width - padding.horizontal(),
                    x + padding.left,
                    inline_y,
                );

                child_cursor.y += inline_height;
                if !inline_nodes.is_empty() {
                    children.extend(inline_nodes);
                }

                child_index = inline_end;
                continue;
            }

            let is_first_child = if let Ok(display) = display {
                child_index == 0 && display.outside() == Some(OutsideDisplay::Block)
            } else {
                false
            };

            let child_ctx = LayoutContext {
                containing_block: Rect {
                    x: x + padding.left + border.left,
                    y: y + padding.top + border.top,
                    width: content_width - padding.horizontal(),
                    height: ctx.containing_block.height,
                },
                parent_padding_top: if is_body { padding.top } else { 0.0 },
                is_first_child,
            };

            let child_node =
                LayoutEngine::layout_node(style_node, &child_ctx, &mut child_cursor, text_ctx);

            if child_node.is_none() {
                // For `display: none`
                child_index += 1;
                continue;
            }

            let child_node = child_node.unwrap();
            let next_sibling = styled_node.children.get(child_index + 1);

            child_cursor.y = Self::calculate_child_y_cursor(
                child_ctx,
                styled_node,
                &child_node,
                &children,
                next_sibling,
                child_cursor.y,
            );

            children.push(child_node);
            child_index += 1;
        }

        content_height += PropertyResolver::calculate_height(
            styled_node,
            ctx.containing_block.height,
            child_cursor.y,
        );

        if styled_node.tag == Some(Tag::Html(HtmlTag::Body)) {
            content_height += Self::body_height_adjustment(
                padding,
                &mut collapsed_margin_top,
                &mut collapsed_margin_bottom,
                &children,
            );
        } else if styled_node.tag == Some(Tag::Html(HtmlTag::Html))
            && let Some(body_node) = children.first()
        {
            content_height = body_node.dimensions.height + Self::html_height_adjustment(body_node);
        } else {
            content_height += padding.vertical() + border.vertical();
        }

        let dimensions = Rect {
            x,
            y,
            width: content_width,
            height: content_height,
        };

        LayoutNode {
            node_id: styled_node.node_id,
            dimensions,
            colors,
            resolved_margin: margin,
            collapsed_margin_bottom,
            collapsed_margin_top,
            resolved_padding: padding,
            text_buffer: None,
            children,
        }
    }

    fn calculate_y_pos(
        node: &StyledNode,
        ctx: &LayoutContext,
        cursor: &BlockCursor,
        margin_top: f32,
        padding_top: f32,
    ) -> f32 {
        let base_y = ctx.containing_block.y + cursor.y;

        if node.tag == Some(Tag::Html(HtmlTag::Body)) && padding_top == 0.0 {
            let mut child_margin_top = 0.0;
            let mut current = node.children.first();
            while let Some(c) = current {
                current = c.children.first();
                let resolved_margin = PropertyResolver::resolve_node_margins(
                    c,
                    ctx.containing_block.width,
                    c.style.computed_font_size_px,
                );

                if resolved_margin.top != 0.0 {
                    child_margin_top = resolved_margin.top;
                    break;
                }
            }

            let collapsed_margin = Self::collapse_margins(margin_top, child_margin_top);
            return base_y + collapsed_margin;
        }

        if let Some(child) = node.children.first() {
            if node.tag == Some(Tag::Html(HtmlTag::Body)) && padding_top != 0.0 {
                return base_y + margin_top;
            }

            if let Ok(display) = Property::resolve(&child.style.display)
                && display.outside() == Some(OutsideDisplay::Inline)
            {
                return base_y;
            }

            let child_margin_top = PropertyResolver::resolve_node_margins(
                child,
                ctx.containing_block.width,
                child.style.computed_font_size_px,
            )
            .top;

            let collapsed_margin = Self::collapse_margins(margin_top, child_margin_top);

            if let Ok(child_margin) = Property::resolve(&child.style.margin)
                && base_y != 0.0
                && base_y != margin_top
                && child_margin.top != OffsetValue::zero()
            {
                return base_y;
            }

            return base_y + collapsed_margin;
        }

        if let Ok(display) = Property::resolve(&node.style.display) {
            if ctx.is_first_child
                && ctx.parent_padding_top == 0.0
                && display.outside() == Some(OutsideDisplay::Block)
            {
                return base_y;
            } else if ctx.is_first_child && display.outside() == Some(OutsideDisplay::Block) {
                return base_y + margin_top;
            }
        }

        base_y
    }

    fn calculate_child_y_cursor(
        ctx: LayoutContext,
        style_node: &StyledNode,
        child_node: &LayoutNode,
        children: &[LayoutNode],
        next_sibling: Option<&StyledNode>,
        child_cursor: f32,
    ) -> f32 {
        if let Ok(display) = Property::resolve(&style_node.style.display)
            && display.outside() != Some(OutsideDisplay::Block)
        {
            return child_node.dimensions.height;
        }

        if let Some(next) = next_sibling {
            let next_margin_top =
                PropertyResolver::resolve_node_margins(next, 0.0, next.style.computed_font_size_px)
                    .top;

            let collapsed_margin =
                Self::collapse_margins(child_node.resolved_margin.bottom, next_margin_top);

            if next_margin_top == collapsed_margin {
                if ctx.is_first_child && ctx.parent_padding_top != 0.0 {
                    return child_cursor
                        + child_node.resolved_margin.top
                        + child_node.dimensions.height
                        + collapsed_margin;
                }

                return child_cursor + child_node.dimensions.height + collapsed_margin;
            }

            return child_cursor + child_node.dimensions.height + collapsed_margin;
        }

        if let Some(previous_sibling) = children.last() {
            let collapsed_margin = Self::collapse_margins(
                child_node.resolved_margin.top,
                previous_sibling.resolved_margin.bottom,
            );

            let y = child_cursor
                + if collapsed_margin > previous_sibling.resolved_margin.bottom {
                    collapsed_margin
                } else {
                    child_node.resolved_margin.top
                };

            return y + child_node.dimensions.height;
        }

        let collapsed_margin = Self::collapse_margins(
            child_node.resolved_margin.bottom,
            child_node.resolved_margin.top,
        );

        child_cursor + child_node.dimensions.height + collapsed_margin
    }

    fn collapse_margins(margin1: f32, margin2: f32) -> f32 {
        if margin1 >= 0.0 && margin2 >= 0.0 {
            f32::max(margin1, margin2)
        } else if margin1 < 0.0 && margin2 < 0.0 {
            f32::min(margin1, margin2)
        } else {
            margin1 + margin2
        }
    }

    fn body_height_adjustment(
        padding: SideOffset,
        collapsed_margin_top: &mut f32,
        collapsed_margin_bottom: &mut f32,
        children: &[LayoutNode],
    ) -> f32 {
        let mut content_height = 0.0;

        let last_child_margin_bottom = {
            let mut current = children.last();
            let mut last_margin = 0.0;

            while let Some(child) = current {
                if child.resolved_margin.bottom != 0.0 {
                    last_margin = child.resolved_margin.bottom;
                    break;
                }
                current = child.children.last();
            }

            last_margin
        };

        let first_child_margin_top = {
            let mut current = children.first();
            let mut first_margin = 0.0;
            while let Some(child) = current {
                if child.resolved_margin.top != 0.0 {
                    first_margin = child.resolved_margin.top;
                    break;
                }
                current = child.children.first();
            }

            first_margin
        };

        if padding.top == 0.0 {
            *collapsed_margin_top = first_child_margin_top;
        } else {
            content_height += padding.top;
        }

        if padding.bottom == 0.0 {
            content_height -= last_child_margin_bottom;
            *collapsed_margin_bottom = last_child_margin_bottom;
        } else {
            content_height += padding.bottom;
        }

        content_height
    }

    fn html_height_adjustment(body_node: &LayoutNode) -> f32 {
        let mut content_height = 0.0;

        if body_node.collapsed_margin_bottom > 0.0 {
            content_height += body_node.collapsed_margin_bottom;
        } else {
            content_height += body_node.resolved_margin.bottom;
        }

        if body_node.collapsed_margin_top > 0.0 {
            content_height += body_node.collapsed_margin_top;
        } else {
            content_height += body_node.resolved_margin.top;
        }

        content_height
    }
}

#[cfg(test)]
mod tests {

    use css_style::{ComputedStyle, Display, display::OutsideDisplay};
    use html_dom::NodeId;

    use super::*;

    fn viewport() -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        }
    }

    #[test]
    fn test_collapsing_margins() {
        assert_eq!(BlockLayout::collapse_margins(10.0, 20.0), 20.0);
        assert_eq!(BlockLayout::collapse_margins(-10.0, -20.0), -20.0);
        assert_eq!(BlockLayout::collapse_margins(10.0, -5.0), 5.0);
        assert_eq!(BlockLayout::collapse_margins(-10.0, 5.0), -5.0);
        assert_eq!(BlockLayout::collapse_margins(0.0, 15.0), 15.0);
        assert_eq!(BlockLayout::collapse_margins(-5.0, 0.0), -5.0);
    }

    #[test]
    fn test_html_height_adjustment_with_all_collapse() {
        let body_node = LayoutNode {
            dimensions: Rect {
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 500.0,
            },
            resolved_margin: SideOffset {
                top: 10.0,
                right: 0.0,
                bottom: 15.0,
                left: 0.0,
            },
            collapsed_margin_top: 5.0,
            collapsed_margin_bottom: 7.0,
            ..LayoutNode::new(NodeId(1))
        };

        let adjustment = BlockLayout::html_height_adjustment(&body_node);
        assert_eq!(adjustment, 12.0);
    }

    #[test]
    fn test_html_height_adjustment_with_top_collapse() {
        let body_node = LayoutNode {
            dimensions: Rect {
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 500.0,
            },
            resolved_margin: SideOffset {
                top: 10.0,
                right: 0.0,
                bottom: 15.0,
                left: 0.0,
            },
            collapsed_margin_top: 5.0,
            collapsed_margin_bottom: 0.0,
            ..LayoutNode::new(NodeId(1))
        };

        let adjustment = BlockLayout::html_height_adjustment(&body_node);
        assert_eq!(adjustment, 20.0);
    }

    #[test]
    fn test_html_height_adjustment_with_bottom_collapse() {
        let body_node = LayoutNode {
            dimensions: Rect {
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 500.0,
            },
            resolved_margin: SideOffset {
                top: 10.0,
                right: 0.0,
                bottom: 15.0,
                left: 0.0,
            },
            collapsed_margin_top: 0.0,
            collapsed_margin_bottom: 7.0,
            ..LayoutNode::new(NodeId(1))
        };

        let adjustment = BlockLayout::html_height_adjustment(&body_node);
        assert_eq!(adjustment, 17.0);
    }

    #[test]
    fn test_body_height_adjustment_no_padding() {
        let padding = SideOffset {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        };
        let mut collapsed_margin_top = 0.0;
        let mut collapsed_margin_bottom = 0.0;
        let children = vec![
            LayoutNode {
                dimensions: Rect::default(),
                resolved_margin: SideOffset {
                    top: 10.0,
                    right: 0.0,
                    bottom: 5.0,
                    left: 0.0,
                },
                ..LayoutNode::new(NodeId(1))
            },
            LayoutNode {
                dimensions: Rect::default(),
                resolved_margin: SideOffset {
                    top: 0.0,
                    right: 0.0,
                    bottom: 15.0,
                    left: 0.0,
                },
                ..LayoutNode::new(NodeId(2))
            },
        ];

        let adjustment = BlockLayout::body_height_adjustment(
            padding,
            &mut collapsed_margin_top,
            &mut collapsed_margin_bottom,
            &children,
        );

        assert_eq!(adjustment, -15.0); // Body height same as the lowest margin bottom
        assert_eq!(collapsed_margin_top, 10.0);
        assert_eq!(collapsed_margin_bottom, 15.0);
    }

    #[test]
    fn test_body_height_adjustment_vertical_padding() {
        let padding = SideOffset {
            top: 5.0,
            right: 0.0,
            bottom: 5.0,
            left: 0.0,
        };
        let mut collapsed_margin_top = 0.0;
        let mut collapsed_margin_bottom = 0.0;
        let children = vec![
            LayoutNode {
                dimensions: Rect::default(),
                resolved_margin: SideOffset {
                    top: 10.0,
                    right: 0.0,
                    bottom: 5.0,
                    left: 0.0,
                },
                ..LayoutNode::new(NodeId(1))
            },
            LayoutNode {
                dimensions: Rect::default(),
                resolved_margin: SideOffset {
                    top: 0.0,
                    right: 0.0,
                    bottom: 15.0,
                    left: 0.0,
                },
                ..LayoutNode::new(NodeId(2))
            },
        ];

        let adjustment = BlockLayout::body_height_adjustment(
            padding,
            &mut collapsed_margin_top,
            &mut collapsed_margin_bottom,
            &children,
        );

        assert_eq!(adjustment, 10.0);
        assert_eq!(collapsed_margin_top, 0.0);
        assert_eq!(collapsed_margin_bottom, 0.0);
    }

    #[test]
    fn test_body_height_adjustment_top_padding() {
        let padding = SideOffset {
            top: 5.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        };
        let mut collapsed_margin_top = 0.0;
        let mut collapsed_margin_bottom = 0.0;
        let children = vec![
            LayoutNode {
                dimensions: Rect::default(),
                resolved_margin: SideOffset {
                    top: 10.0,
                    right: 0.0,
                    bottom: 5.0,
                    left: 0.0,
                },
                ..LayoutNode::new(NodeId(1))
            },
            LayoutNode {
                dimensions: Rect::default(),
                resolved_margin: SideOffset {
                    top: 0.0,
                    right: 0.0,
                    bottom: 15.0,
                    left: 0.0,
                },
                ..LayoutNode::new(NodeId(2))
            },
        ];

        let adjustment = BlockLayout::body_height_adjustment(
            padding,
            &mut collapsed_margin_top,
            &mut collapsed_margin_bottom,
            &children,
        );

        assert_eq!(adjustment, -10.0);
        assert_eq!(collapsed_margin_top, 0.0);
        assert_eq!(collapsed_margin_bottom, 15.0);
    }

    #[test]
    fn test_body_height_adjustment_bottom_padding() {
        let padding = SideOffset {
            top: 0.0,
            right: 0.0,
            bottom: 5.0,
            left: 0.0,
        };
        let mut collapsed_margin_top = 0.0;
        let mut collapsed_margin_bottom = 0.0;
        let children = vec![
            LayoutNode {
                dimensions: Rect::default(),
                resolved_margin: SideOffset {
                    top: 10.0,
                    right: 0.0,
                    bottom: 5.0,
                    left: 0.0,
                },
                ..LayoutNode::new(NodeId(1))
            },
            LayoutNode {
                dimensions: Rect::default(),
                resolved_margin: SideOffset {
                    top: 0.0,
                    right: 0.0,
                    bottom: 15.0,
                    left: 0.0,
                },
                ..LayoutNode::new(NodeId(2))
            },
        ];

        let adjustment = BlockLayout::body_height_adjustment(
            padding,
            &mut collapsed_margin_top,
            &mut collapsed_margin_bottom,
            &children,
        );

        assert_eq!(adjustment, 5.0);
        assert_eq!(collapsed_margin_top, 10.0);
        assert_eq!(collapsed_margin_bottom, 0.0);
    }

    #[test]
    fn test_calculate_y_pos_not_first_child() {
        let styled_node = StyledNode {
            style: ComputedStyle {
                display: Property::from(Display::from(OutsideDisplay::Block)),
                computed_font_size_px: 16.0,
                ..Default::default()
            },
            ..StyledNode::new(NodeId(1))
        };

        let ctx = LayoutContext {
            containing_block: viewport(),
            parent_padding_top: 10.0,
            is_first_child: false,
        };

        let cursor = BlockCursor { y: 0.0 };
        let margin_top = 20.0;
        let padding_top = 0.0;

        let y_pos =
            BlockLayout::calculate_y_pos(&styled_node, &ctx, &cursor, margin_top, padding_top);

        assert_eq!(y_pos, 0.0);
    }

    #[test]
    fn test_calculate_y_pos_first_child_no_padding() {
        let styled_node = StyledNode {
            style: ComputedStyle {
                display: Property::from(Display::from(OutsideDisplay::Block)),
                computed_font_size_px: 16.0,
                ..Default::default()
            },
            ..StyledNode::new(NodeId(1))
        };

        let ctx = LayoutContext {
            containing_block: viewport(),
            parent_padding_top: 0.0,
            is_first_child: true,
        };

        let cursor = BlockCursor { y: 0.0 };
        let margin_top = 20.0;
        let padding_top = 0.0;

        let y_pos =
            BlockLayout::calculate_y_pos(&styled_node, &ctx, &cursor, margin_top, padding_top);

        assert_eq!(y_pos, 0.0);
    }

    #[test]
    fn test_calculate_y_pos_first_child_padding() {
        let styled_node = StyledNode {
            style: ComputedStyle {
                display: Property::from(Display::from(OutsideDisplay::Block)),
                computed_font_size_px: 16.0,
                ..Default::default()
            },
            ..StyledNode::new(NodeId(1))
        };

        let ctx = LayoutContext {
            containing_block: viewport(),
            parent_padding_top: 10.0,
            is_first_child: true,
        };

        let cursor = BlockCursor { y: 0.0 };
        let margin_top = 20.0;
        let padding_top = 0.0;

        let y_pos =
            BlockLayout::calculate_y_pos(&styled_node, &ctx, &cursor, margin_top, padding_top);

        assert_eq!(y_pos, 20.0);
    }

    #[test]
    fn test_calculate_y_pos_with_child() {
        let child_node = StyledNode {
            style: ComputedStyle {
                display: Property::from(Display::from(OutsideDisplay::Block)),
                computed_font_size_px: 16.0,
                ..Default::default()
            },
            ..StyledNode::new(NodeId(2))
        };
        let parent_node = StyledNode {
            style: ComputedStyle {
                display: Property::from(Display::from(OutsideDisplay::Block)),
                computed_font_size_px: 16.0,
                ..Default::default()
            },
            children: vec![child_node],
            ..StyledNode::new(NodeId(1))
        };
        let ctx = LayoutContext {
            containing_block: viewport(),
            parent_padding_top: 0.0,
            is_first_child: false,
        };
        let cursor = BlockCursor { y: 0.0 };
        let margin_top = 20.0;
        let padding_top = 0.0;

        let y_pos =
            BlockLayout::calculate_y_pos(&parent_node, &ctx, &cursor, margin_top, padding_top);
        assert_eq!(y_pos, 20.0);
    }

    #[test]
    fn test_calculate_y_pos_body_with_padding() {
        let child_node = StyledNode {
            style: ComputedStyle {
                display: Property::from(Display::from(OutsideDisplay::Block)),
                computed_font_size_px: 16.0,
                ..Default::default()
            },
            ..StyledNode::new(NodeId(2))
        };
        let parent_node = StyledNode {
            tag: Some(Tag::Html(HtmlTag::Body)),
            style: ComputedStyle {
                display: Property::from(Display::from(OutsideDisplay::Block)),
                computed_font_size_px: 16.0,
                ..Default::default()
            },
            children: vec![child_node],
            ..StyledNode::new(NodeId(1))
        };
        let ctx = LayoutContext {
            containing_block: viewport(),
            parent_padding_top: 0.0,
            is_first_child: false,
        };
        let cursor = BlockCursor { y: 0.0 };
        let margin_top = 20.0;
        let padding_top = 10.0;

        let y_pos =
            BlockLayout::calculate_y_pos(&parent_node, &ctx, &cursor, margin_top, padding_top);
        assert_eq!(y_pos, 20.0);
    }

    #[test]
    fn test_calculate_y_pos_body_without_padding() {
        let child_node = StyledNode {
            style: ComputedStyle {
                display: Property::from(Display::from(OutsideDisplay::Block)),
                computed_font_size_px: 16.0,
                ..Default::default()
            },
            ..StyledNode::new(NodeId(2))
        };
        let parent_node = StyledNode {
            tag: Some(Tag::Html(HtmlTag::Body)),
            style: ComputedStyle {
                display: Property::from(Display::from(OutsideDisplay::Block)),
                computed_font_size_px: 16.0,
                ..Default::default()
            },
            children: vec![child_node],
            ..StyledNode::new(NodeId(1))
        };
        let ctx = LayoutContext {
            containing_block: viewport(),
            parent_padding_top: 0.0,
            is_first_child: false,
        };
        let cursor = BlockCursor { y: 0.0 };
        let margin_top = 20.0;
        let padding_top = 0.0;

        let y_pos =
            BlockLayout::calculate_y_pos(&parent_node, &ctx, &cursor, margin_top, padding_top);
        assert_eq!(y_pos, 20.0);
    }

    #[test]
    fn test_calculate_y_pos_body_first_child() {
        let child_node = StyledNode {
            style: ComputedStyle {
                display: Property::from(Display::from(OutsideDisplay::Block)),
                computed_font_size_px: 16.0,
                ..Default::default()
            },
            ..StyledNode::new(NodeId(2))
        };
        let parent_node = StyledNode {
            tag: Some(Tag::Html(HtmlTag::Body)),
            style: ComputedStyle {
                display: Property::from(Display::from(OutsideDisplay::Block)),
                computed_font_size_px: 16.0,
                ..Default::default()
            },
            children: vec![child_node],
            ..StyledNode::new(NodeId(1))
        };
        let ctx = LayoutContext {
            containing_block: viewport(),
            parent_padding_top: 0.0,
            is_first_child: true,
        };
        let cursor = BlockCursor { y: 0.0 };
        let margin_top = 20.0;
        let padding_top = 0.0;

        let y_pos =
            BlockLayout::calculate_y_pos(&parent_node, &ctx, &cursor, margin_top, padding_top);
        assert_eq!(y_pos, 20.0);
    }

    #[test]
    fn test_layout_empty() {
        let styled_node = StyledNode {
            style: ComputedStyle {
                display: Property::from(Display::from(OutsideDisplay::Block)),
                ..Default::default()
            },
            ..StyledNode::new(NodeId(0))
        };

        let ctx = LayoutContext {
            containing_block: viewport(),
            parent_padding_top: 0.0,
            is_first_child: true,
        };

        let mut cursor = BlockCursor { y: 0.0 };
        let mut text_ctx = TextContext::default();

        let layout_node = BlockLayout::layout(&styled_node, &ctx, &mut cursor, &mut text_ctx);

        assert_eq!(layout_node.node_id, styled_node.node_id);
        assert_eq!(layout_node.dimensions.x, 0.0);
        assert_eq!(layout_node.dimensions.y, 0.0);
        assert_eq!(layout_node.dimensions.width, 800.0);
        assert_eq!(layout_node.dimensions.height, 0.0);
        assert_eq!(layout_node.children.len(), 0);
    }
}
