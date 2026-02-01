use std::sync::Arc;

use css_style::{ComputedStyle, StyledNode, types::display::OutsideDisplay};
use html_dom::{HtmlTag, NodeId, Tag};

use crate::{
    Color4f, LayoutColors, LayoutNode, Rect, TextContext,
    resolver::PropertyResolver,
    text::{TextDescription, TextOffsetContext},
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
        style: Box<ComputedStyle>,
    },
    Break {
        font_size_px: f32,
    },
}

pub struct InlineLayout;

impl InlineLayout {
    pub fn collect_inline_items_from_nodes(nodes: &[StyledNode]) -> Vec<InlineItem> {
        let mut items = Vec::new();

        for node in nodes {
            let result = Self::collect(node, &mut items);

            if result.is_err() {
                break;
            }
        }

        items
    }

    fn collect(inline_node: &StyledNode, items: &mut Vec<InlineItem>) -> Result<(), ()> {
        if let Some(text) = inline_node.text_content.as_ref() {
            items.push(InlineItem::TextRun {
                id: inline_node.node_id,
                text: text.clone(),
                style: Box::new(inline_node.style.clone()),
            });
        }

        if let Some(tag) = inline_node.tag.as_ref() {
            if inline_node.style.display.outside != Some(OutsideDisplay::Inline)
                && !items.is_empty()
            {
                return Err(());
            }

            match tag {
                Tag::Html(HtmlTag::Br) => {
                    let font_size_px = inline_node.style.computed_font_size_px;
                    items.push(InlineItem::Break { font_size_px });
                }
                _ => {
                    if !inline_node.children.is_empty() {
                        for child in &inline_node.children {
                            let result = Self::collect(child, items);

                            if result.is_err() {
                                return Err(());
                            }
                        }
                    }
                }
            }
        }

        Ok(())
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
                    let text_align = style.text_align;
                    let line_height = &style.line_height;
                    let font_family = &style.font_family;
                    let font_weight = &style.font_weight;

                    let (i_text, r_text) = text_ctx.measure_multiline_text(
                        text,
                        &TextDescription {
                            font_size_px,
                            font_family,
                            font_weight,
                            line_height,
                        },
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

                    let first_y = y + cursor.y;
                    total_height = i_text.height;

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
                        collapsed_margin_top: 0.0,
                        collapsed_margin_bottom: 0.0,
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
                        total_height += r_text.height - line_height_px;

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
                            collapsed_margin_top: 0.0,
                            collapsed_margin_bottom: 0.0,
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
                InlineItem::Break { font_size_px } => {
                    let line_height_px = *font_size_px * 1.2;
                    cursor.y += line_height_px;
                    cursor.x = 0.0;
                    cursor.remaining_width = width;
                    total_height += line_height_px;
                }
            }
        }

        (nodes, total_height)
    }
}

#[cfg(test)]
mod tests {
    use assets::{
        ASSETS,
        constants::{DEFAULT_FONT, MONOSPACE_FONT},
    };
    use cosmic_text::{FontSystem, fontdb::Source};

    use super::*;

    #[test]
    fn test_collect_inline_items() {
        let node_text = StyledNode {
            text_content: Some(String::from("Hello, world!")),
            ..StyledNode::new(NodeId(0))
        };

        let node_break = StyledNode {
            tag: Some(Tag::Html(HtmlTag::Br)),
            style: ComputedStyle {
                display: css_style::types::display::Display {
                    outside: Some(OutsideDisplay::Inline),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..StyledNode::new(NodeId(1))
        };

        let nodes = vec![node_text, node_break];
        let items = InlineLayout::collect_inline_items_from_nodes(&nodes);

        assert_eq!(items.len(), 2);
        match &items[0] {
            InlineItem::TextRun { text, .. } => assert_eq!(text, "Hello, world!"),
            _ => panic!("Expected TextRun"),
        }
        match &items[1] {
            InlineItem::Break { .. } => (),
            _ => panic!("Expected Break"),
        }
    }

    #[test]
    fn test_collect_inline_items_with_block() {
        let node_text = StyledNode {
            text_content: Some(String::from("Hello, world!")),
            ..StyledNode::new(NodeId(0))
        };

        let node_block = StyledNode {
            tag: Some(Tag::Html(HtmlTag::Div)),
            style: ComputedStyle {
                display: css_style::types::display::Display {
                    outside: Some(OutsideDisplay::Block),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..StyledNode::new(NodeId(2))
        };

        let node_break = StyledNode {
            tag: Some(Tag::Html(HtmlTag::Br)),
            style: ComputedStyle {
                display: css_style::types::display::Display {
                    outside: Some(OutsideDisplay::Inline),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..StyledNode::new(NodeId(1))
        };

        let nodes = vec![node_text, node_block, node_break];
        let items = InlineLayout::collect_inline_items_from_nodes(&nodes);

        assert_eq!(items.len(), 1);
        match &items[0] {
            InlineItem::TextRun { text, .. } => assert_eq!(text, "Hello, world!"),
            _ => panic!("Expected TextRun"),
        }
    }

    #[test]
    fn test_inline_layout() {
        let items = vec![
            InlineItem::TextRun {
                id: NodeId(0),
                text: String::from("Hello, world! This is a test of inline layout."),
                style: Box::new(ComputedStyle::default()),
            },
            InlineItem::Break { font_size_px: 16.0 },
            InlineItem::TextRun {
                id: NodeId(1),
                text: String::from("This is the second line after a break."),
                style: Box::new(ComputedStyle::default()),
            },
        ];

        let mut text_ctx = TextContext::default();
        let (layout_nodes, total_height) =
            InlineLayout::layout(&items, &mut text_ctx, 200.0, 0.0, 0.0);

        assert_eq!(layout_nodes.len(), 2);
        assert!(total_height > 16.0);
    }

    #[test]
    fn test_inline_layout_multiline_text() {
        let default_font = ASSETS.read().unwrap().load_embedded(DEFAULT_FONT);
        let monospace_font = ASSETS.read().unwrap().load_embedded(MONOSPACE_FONT);

        let mut text_context = TextContext::new(FontSystem::new_with_fonts(vec![
            Source::Binary(Arc::new(default_font)),
            Source::Binary(Arc::new(monospace_font)),
        ]));

        let items = vec![
            InlineItem::TextRun {
                id: NodeId(0),
                text: String::from("A very short line."),
                style: Box::new(ComputedStyle::default()),
            },
            InlineItem::TextRun {
                id: NodeId(1),
                text: String::from(
                    "Hello, world! This is a test of inline layout. This text should wrap to the next line, because it is too long to fit in the given width.",
                ),
                style: Box::new(ComputedStyle::default()),
            },
        ];

        let (layout_nodes, total_height) =
            InlineLayout::layout(&items, &mut text_context, 50.0, 0.0, 0.0);

        assert_eq!(layout_nodes.len(), 3);
        assert!(total_height > 16.0);
    }
}
