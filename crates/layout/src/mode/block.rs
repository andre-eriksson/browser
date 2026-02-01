use css_style::{StyledNode, types::display::OutsideDisplay};
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

        let colors = LayoutColors {
            background_color: Color4f::from_css_color(&styled_node.style.background_color),
            color: Color4f::from_css_color(&styled_node.style.color),
        };

        let x = ctx.containing_block.x + margin.left;
        let y = Self::calculate_y_pos(styled_node, ctx, cursor, margin.top, padding.top);

        let content_width =
            PropertyResolver::calculate_width(styled_node, ctx.containing_block.width, &margin);

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

            if style_node.style.display.outside == Some(OutsideDisplay::Inline) {
                let mut inline_end = child_index + 1;
                while inline_end < child_len
                    && styled_node.children[inline_end].style.display.outside
                        == Some(OutsideDisplay::Inline)
                {
                    inline_end += 1;
                }

                let items = InlineLayout::collect_inline_items_from_nodes(
                    &styled_node.children[child_index..inline_end],
                );

                let inline_y = y + child_cursor.y + padding.top;
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

            let is_first_child =
                child_index == 0 && style_node.style.display.outside == Some(OutsideDisplay::Block);

            let child_ctx = LayoutContext {
                containing_block: Rect {
                    x: x + padding.left,
                    y: y + padding.top,
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
            content_height += padding.vertical();
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

        if let Some(child) = node.children.first() {
            if node.tag == Some(Tag::Html(HtmlTag::Body)) && padding_top != 0.0 {
                return base_y + margin_top;
            }

            let child_margin_top = PropertyResolver::resolve_node_margins(
                child,
                ctx.containing_block.width,
                child.style.computed_font_size_px,
            )
            .top;

            let collapsed_margin = Self::collapse_margins(margin_top, child_margin_top);

            return base_y + collapsed_margin;
        }

        if ctx.is_first_child
            && ctx.parent_padding_top == 0.0
            && node.style.display.outside == Some(OutsideDisplay::Block)
        {
            return base_y;
        } else if ctx.is_first_child && node.style.display.outside == Some(OutsideDisplay::Block) {
            return base_y + margin_top;
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
        if style_node.style.display.outside == Some(OutsideDisplay::Block) {
            if let Some(next) = next_sibling {
                let next_margin_top = PropertyResolver::resolve_node_margins(
                    next,
                    0.0,
                    next.style.computed_font_size_px,
                )
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

            return child_cursor + child_node.dimensions.height + child_node.resolved_margin.top;
        }

        child_node.dimensions.height
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

        let last_child_margin_bottom = if let Some(last_child) = children.last() {
            last_child.resolved_margin.bottom
        } else {
            0.0
        };

        let first_child_margin_top = if let Some(first_child) = children.first() {
            first_child.resolved_margin.top
        } else {
            0.0
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
