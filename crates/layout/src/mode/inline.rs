use std::sync::Arc;

use css_style::{
    ComputedStyle, LineHeight, Property, StyledNode, TextAlign, Whitespace, display::OutsideDisplay,
};
use html_dom::{HtmlTag, NodeId, Tag};

use crate::{
    LayoutColors, LayoutNode, Rect, TextContext,
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
        line_height_px: f32,
    },
}

pub struct InlineContext {
    pub inside_preformatted: bool,
}

pub struct InlineLayout;

impl InlineLayout {
    pub fn collect_inline_items_from_nodes(
        parent_style: &ComputedStyle,
        nodes: &[StyledNode],
    ) -> Vec<InlineItem> {
        let mut items = Vec::new();
        let mut ctx = InlineContext {
            inside_preformatted: false,
        };

        for node in nodes {
            let result = Self::collect(&mut ctx, parent_style, node, &mut items);

            if result.is_err() {
                break;
            }
        }

        items
    }

    fn collect(
        ctx: &mut InlineContext,
        style: &ComputedStyle,
        inline_node: &StyledNode,
        items: &mut Vec<InlineItem>,
    ) -> Result<(), ()> {
        if let Some(text) = inline_node.text_content.as_ref() {
            let inherited_styles = style.inherited_subset();

            if let Ok(whitespace) = Property::resolve(&inherited_styles.whitespace) {
                ctx.inside_preformatted =
                    matches!(whitespace, Whitespace::Pre | Whitespace::PreWrap);

                let adjusted_text = Self::preserve_significant_whitespace(ctx, text);

                match whitespace {
                    Whitespace::Normal => {
                        let collapsed = adjusted_text
                            .split_whitespace()
                            .collect::<Vec<_>>()
                            .join(" ");

                        if !collapsed.is_empty() {
                            items.push(InlineItem::TextRun {
                                id: inline_node.node_id,
                                text: collapsed,
                                style: Box::new(inherited_styles.clone()),
                            });
                        }
                    }
                    Whitespace::PreLine => {
                        let collapsed = adjusted_text
                            .split_whitespace()
                            .collect::<Vec<_>>()
                            .join(" ");

                        items.push(InlineItem::TextRun {
                            id: inline_node.node_id,
                            text: collapsed,
                            style: Box::new(inherited_styles.clone()),
                        });
                    }
                    Whitespace::PreWrap | Whitespace::Pre => {
                        let mut text = adjusted_text;
                        if items.is_empty() {
                            text = text.trim_start_matches('\n').to_string();
                        }

                        let lines: Vec<&str> = text.split('\n').collect();

                        for (i, line) in lines.iter().enumerate() {
                            if !line.is_empty() || i == 0 {
                                items.push(InlineItem::TextRun {
                                    id: inline_node.node_id,
                                    text: line.to_string(),
                                    style: Box::new(inherited_styles.clone()),
                                });
                            }

                            if i < lines.len() - 1 {
                                let font_size = inline_node.style.computed_font_size_px;
                                items.push(InlineItem::Break {
                                    line_height_px: Property::resolve(&style.line_height)
                                        .map_or(LineHeight::default().to_px(font_size), |lh| {
                                            lh.to_px(font_size)
                                        }),
                                });
                            }
                        }
                    }
                }
            } else {
                return Err(());
            }
        }

        if let Some(tag) = inline_node.tag.as_ref() {
            if let Ok(display) = Property::resolve(&inline_node.style.display)
                && display.outside() != Some(OutsideDisplay::Inline)
                && !items.is_empty()
            {
                return Err(());
            }

            match tag {
                Tag::Html(HtmlTag::Br) => {
                    let font_size = inline_node.style.computed_font_size_px;
                    items.push(InlineItem::Break {
                        line_height_px: Property::resolve(&inline_node.style.line_height)
                            .map_or(LineHeight::default().to_px(font_size), |lh| {
                                lh.to_px(font_size)
                            }),
                    });
                }
                _ => {
                    if !inline_node.children.is_empty() {
                        for child in &inline_node.children {
                            let result = Self::collect(ctx, &inline_node.style, child, items);

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

                    let (
                        Ok(whitespace),
                        Ok(text_align),
                        Ok(line_height),
                        Ok(font_family),
                        Ok(font_weight),
                    ) = (
                        Property::resolve(&style.whitespace),
                        Property::resolve(&style.text_align),
                        Property::resolve(&style.line_height),
                        Property::resolve(&style.font_family),
                        Property::resolve(&style.font_weight),
                    )
                    else {
                        continue;
                    };

                    let (i_text, r_text) = text_ctx.measure_multiline_text(
                        text,
                        &TextDescription {
                            whitespace,
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

                    let (margin, padding, _border) =
                        PropertyResolver::resolve_box_model(style, width, font_size_px);

                    let colors = LayoutColors::from(style);

                    let mut first_x = x + cursor.x + margin.left;

                    match text_align {
                        TextAlign::Center => {
                            let total_line_width = i_text.width + margin.horizontal();
                            first_x = x + (width - total_line_width) / 2.0 + margin.left + cursor.x;
                        }
                        TextAlign::Right => {
                            let total_line_width = i_text.width + margin.horizontal();
                            first_x = x + width - total_line_width + margin.left;
                        }
                        _ => {}
                    }

                    let first_y = y + cursor.y;
                    total_height = f32::max(total_height, i_text.height + cursor.y);

                    let node = LayoutNode {
                        node_id: *id,
                        dimensions: Rect::new(first_x, first_y, i_text.width, i_text.height),
                        colors: colors.clone(),
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

                        let rest_x = x + margin.left;
                        let rest_y = y + cursor.y + margin.top + padding.top;
                        total_height += r_text.height - line_height_px;

                        let node = LayoutNode {
                            node_id: *id,
                            dimensions: Rect::new(rest_x, rest_y, r_text.width, r_text.height),
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
                InlineItem::Break { line_height_px } => {
                    cursor.y += line_height_px;
                    cursor.x = 0.0;
                    cursor.remaining_width = width;
                    total_height += line_height_px;
                }
            }
        }

        (nodes, total_height)
    }

    pub fn preserve_significant_whitespace(ctx: &InlineContext, text: &str) -> String {
        if ctx.inside_preformatted {
            return text.to_string();
        }

        let has_leading_ws = text.starts_with(char::is_whitespace);
        let has_trailing_ws = text.ends_with(char::is_whitespace);
        let has_leading_newline = text.starts_with('\n') || text.starts_with('\r');
        let has_trailing_newline = text.ends_with('\n') || text.ends_with('\r');

        let normalized_middle = text.split_whitespace().collect::<Vec<_>>().join(" ");

        let mut result = String::new();
        if has_leading_ws && !normalized_middle.is_empty() && !has_leading_newline {
            result.push(' ');
        }
        result.push_str(&normalized_middle);
        if has_trailing_ws && !normalized_middle.is_empty() && !has_trailing_newline {
            result.push(' ');
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use assets::{ASSETS, constants::OPEN_SANS_REGULAR};
    use cosmic_text::{FontSystem, fontdb::Source};
    use css_style::Display;

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
                display: Property::from(Display::from(OutsideDisplay::Inline)),
                ..Default::default()
            },
            ..StyledNode::new(NodeId(1))
        };

        let style = Box::new(ComputedStyle::default());
        let nodes = vec![node_text, node_break];
        let items = InlineLayout::collect_inline_items_from_nodes(&style, &nodes);

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
                display: Property::from(Display::from(OutsideDisplay::Block)),
                ..Default::default()
            },
            ..StyledNode::new(NodeId(2))
        };

        let node_break = StyledNode {
            tag: Some(Tag::Html(HtmlTag::Br)),
            style: ComputedStyle {
                display: Property::from(Display::from(OutsideDisplay::Inline)),
                ..Default::default()
            },
            ..StyledNode::new(NodeId(1))
        };

        let style = Box::new(ComputedStyle::default());
        let nodes = vec![node_text, node_block, node_break];
        let items = InlineLayout::collect_inline_items_from_nodes(&style, &nodes);

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
            InlineItem::Break {
                line_height_px: 1.5,
            },
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
        let default_font = ASSETS.read().unwrap().load_embedded(OPEN_SANS_REGULAR);

        let mut text_context = TextContext::new(FontSystem::new_with_fonts(vec![Source::Binary(
            Arc::new(default_font),
        )]));

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
