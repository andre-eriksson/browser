use css_style::{
    StyledNode,
    types::display::{BoxDisplay, OutsideDisplay},
};
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
        if styled_node.style.display.box_display == Some(BoxDisplay::None) {
            return LayoutNode {
                node_id: styled_node.node_id,
                dimensions: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                },
                colors: LayoutColors::default(),
                resolved_margin: SideOffset::zero(),
                collapsed_margin_bottom: 0.0,
                collapsed_margin_top: 0.0,
                resolved_padding: SideOffset::zero(),
                text_buffer: None,
                children: vec![],
            };
        }

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
        let mut y = ctx.containing_block.y + cursor.y + margin.top;

        if ctx.margin.top != ctx.containing_block.y && ctx.padding.top > 0.0 {
            y -= ctx.containing_block.y;
        }

        if ctx.margin.top == ctx.containing_block.y && margin.top > ctx.margin.top {
            y -= ctx.margin.top;
        }

        let content_width =
            PropertyResolver::calculate_width(styled_node, ctx.containing_block.width, &margin);

        let mut collapsed_margin_top = 0.0;
        let mut collapsed_margin_bottom = 0.0;

        let child_ctx = LayoutContext {
            containing_block: Rect {
                x: x + padding.left,
                y: y + padding.top,
                width: content_width - padding.horizontal(),
                height: ctx.containing_block.height,
            },
            margin,
            padding,
        };

        let mut children = Vec::new();

        let mut content_height = 0.0;
        let mut child_cursor = BlockCursor { y: 0.0 };

        let mut child_index = 0;
        let child_len = styled_node.children.len();

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

                let inline_y = y + child_cursor.y;
                let (inline_nodes, inline_height) = InlineLayout::layout(
                    &items,
                    text_ctx,
                    child_ctx.containing_block.width,
                    x,
                    inline_y,
                );

                child_cursor.y += inline_height;
                if !inline_nodes.is_empty() {
                    children.extend(inline_nodes);
                }

                child_index = inline_end;
                continue;
            }

            let child_node =
                LayoutEngine::layout_node(style_node, &child_ctx, &mut child_cursor, text_ctx);

            if child_node.is_none() {
                // For `display: none`
                child_index += 1;
                continue;
            }

            let child_node = child_node.unwrap();

            if style_node.style.display.outside == Some(OutsideDisplay::Block) {
                // Collapse margins with previous sibling
                let collapsed_margin_top = if let Some(previous_sibling) = children.last() {
                    f32::max(
                        previous_sibling.resolved_margin.bottom,
                        child_node.resolved_margin.top,
                    )
                } else {
                    child_node.resolved_margin.top
                };

                child_cursor.y += child_node.dimensions.height + collapsed_margin_top;
            }

            children.push(child_node);
            child_index += 1;
        }

        content_height += PropertyResolver::calculate_height(
            styled_node,
            ctx.containing_block.height,
            child_cursor.y,
        );

        if styled_node.tag == Some(Tag::Html(HtmlTag::Body)) {
            let first_child_margin_top = if let Some(first_child) = children.first() {
                first_child.resolved_margin.top
            } else {
                0.0
            };

            let last_child_margin_bottom = if let Some(last_child) = children.last() {
                last_child.resolved_margin.bottom
            } else {
                0.0
            };

            if padding.top == 0.0 {
                y = first_child_margin_top;
                collapsed_margin_top = first_child_margin_top;
            } else {
                y = padding.top;
            }

            if padding.bottom == 0.0 {
                content_height -= last_child_margin_bottom;
                collapsed_margin_bottom = last_child_margin_bottom;
            } else {
                content_height += last_child_margin_bottom;
            }
        } else if styled_node.tag == Some(Tag::Html(HtmlTag::Html))
            && let Some(body_node) = children.first()
        {
            content_height = body_node.dimensions.height;
            if body_node.collapsed_margin_top > 0.0 {
                content_height += body_node.collapsed_margin_top;
            } else {
                content_height += body_node.resolved_margin.top;
            }

            if body_node.collapsed_margin_bottom > 0.0 {
                content_height += body_node.collapsed_margin_bottom;
            } else {
                content_height += body_node.resolved_margin.bottom;
            }

            content_height += body_node.resolved_padding.vertical()
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
}
