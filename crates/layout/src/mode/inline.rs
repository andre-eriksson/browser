use std::sync::Arc;

use css_style::{ComputedStyle, StyledNode};
use html_dom::NodeId;

use crate::{
    Color4f, LayoutColors, LayoutNode, Rect, TextContext, resolver::PropertyResolver,
    text::TextOffsetContext,
};

#[derive(Debug, Clone)]
pub struct InlineCursor {
    pub x: f32,
    pub y: f32,
    pub remaining_width: f32,
}

#[derive(Debug, Clone)]
pub enum InlineItem {
    TextRun {
        id: NodeId,
        text: String,
        style: ComputedStyle,
    },
}

pub struct InlineLayout;

impl InlineLayout {
    pub fn collect_inline_items(inline_node: &StyledNode) -> Vec<InlineItem> {
        let mut items = Vec::new();

        if let Some(text) = inline_node.text_content.as_ref() {
            items.push(InlineItem::TextRun {
                id: inline_node.node_id,
                text: text.clone(),
                style: inline_node.style.clone(),
            });
        }

        for child in &inline_node.children {
            let mut child_items = Self::collect_inline_items(child);
            items.append(&mut child_items);
        }

        items
    }

    pub fn layout(
        items: &[InlineItem],
        text_ctx: &mut TextContext,
        width: f32,
        x: f32,
        y: f32,
    ) -> (Vec<LayoutNode>, f32) {
        let mut nodes = Vec::new();
        let mut total_height = 0.0;
        let mut cursor = InlineCursor {
            x: 0.0,
            y: 0.0,
            remaining_width: width,
        };

        for item in items {
            match item {
                InlineItem::TextRun { id, text, style } => {
                    let font_size_px = style.computed_font_size_px;
                    let line_height = &style.line_height;
                    let font_family = &style.font_family;

                    let (i_text, r_text) = text_ctx.measure_multiline_text(
                        text,
                        font_size_px,
                        line_height,
                        font_family,
                        width,
                        TextOffsetContext {
                            available_width: cursor.remaining_width,
                            offset_x: cursor.x,
                        },
                    );

                    let margin =
                        PropertyResolver::resolve_margins(&style.margin, width, font_size_px);

                    let padding =
                        PropertyResolver::resolve_padding(&style.padding, width, font_size_px);

                    let colors = LayoutColors {
                        background_color: Color4f::from_css_color(&style.background_color),
                        color: Color4f::from_css_color(&style.color),
                    };

                    let first_x = x + cursor.x + margin.left + padding.left;
                    let first_y = y + cursor.y + margin.top + padding.top;
                    total_height += i_text.height + margin.vertical() + padding.vertical();

                    let node = LayoutNode {
                        node_id: *id,
                        dimensions: Rect {
                            x: first_x,
                            y: first_y,
                            width: i_text.width,
                            height: i_text.height,
                        },
                        colors,
                        resolved_margin: margin,
                        resolved_padding: padding,
                        text_buffer: Some(Arc::new(i_text.buffer)),
                        children: vec![],
                    };

                    nodes.push(node);

                    if let Some(r_text) = r_text {
                        let line_height_px = line_height.to_px(font_size_px);
                        cursor.y += line_height_px;
                        cursor.x = 0.0;

                        let rest_x = x + margin.left + padding.left;
                        let rest_y = y + cursor.y + margin.top + padding.top;
                        total_height += r_text.height + margin.vertical() + padding.vertical();

                        let node = LayoutNode {
                            node_id: *id,
                            dimensions: Rect {
                                x: rest_x,
                                y: rest_y,
                                width: r_text.width,
                                height: r_text.height,
                            },
                            colors,
                            resolved_margin: margin,
                            resolved_padding: padding,
                            text_buffer: Some(Arc::new(r_text.buffer)),
                            children: vec![],
                        };

                        nodes.push(node);

                        cursor.y +=
                            (r_text.height - line_height_px) + margin.bottom + padding.bottom;
                        cursor.x = r_text.last_line_width + margin.right + padding.right;
                    } else {
                        let line_height_px = line_height.to_px(font_size_px);

                        if i_text.height > line_height_px + 0.1 {
                            cursor.y += i_text.height - line_height_px;
                            cursor.x = i_text.last_line_width + margin.right + padding.right;
                        } else {
                            cursor.x += i_text.last_line_width + margin.right + padding.right;
                        }
                    }

                    cursor.remaining_width = width - cursor.x;
                }
            }
        }

        (nodes, total_height)
    }
}
