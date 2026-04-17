use std::sync::Arc;

use css_style::ComputedStyle;
use css_values::text::Whitespace;
use html_dom::NodeId;

use crate::{
    LayoutColors, LayoutNode, Rect, TextContext,
    float::FloatContext,
    mode::inline::{InlineLayoutContext, collection::TextRun, line::LineBoxBuilder},
    text::TextDescription,
};

struct Text<'text> {
    content: &'text str,
    node_id: NodeId,
    style: &'text ComputedStyle,
    text_desc: &'text TextDescription<'text>,
}

pub fn layout_text<'node>(
    ctx: &mut InlineLayoutContext<'node>,
    float_ctx: &FloatContext,
    text_ctx: &mut TextContext,
    line: &mut LineBoxBuilder<'node>,
    text: &TextRun,
) {
    let font_size_px = text.style.font_size;
    let whitespace = &text.style.whitespace;
    let text_align = text.style.text_align;
    let line_height = text.style.line_height;
    let font_family = &text.style.font_family;
    let font_weight = text.style.font_weight;
    let writing_mode = &text.style.writing_mode;

    text_ctx.last_text_align = text_align;
    text_ctx.last_writing_mode = *writing_mode;

    let preserves_newlines = matches!(whitespace, Whitespace::Pre | Whitespace::PreWrap | Whitespace::PreLine);

    let text_desc = TextDescription {
        whitespace,
        line_height,
        font_family,
        font_weight,
        font_size_px,
    };

    if preserves_newlines && text.content.contains('\n') {
        let segments: Vec<&str> = text.content.split('\n').collect();

        for (seg_idx, segment) in segments.iter().enumerate() {
            if !segment.is_empty() {
                layout_text_segment(
                    ctx,
                    text_ctx,
                    float_ctx,
                    &Text {
                        content: segment,
                        node_id: text.id,
                        style: text.style,
                        text_desc: &text_desc,
                    },
                    line,
                );
            }

            if seg_idx < segments.len() - 1 {
                line.finish_line_with_decorations(ctx, text_ctx, float_ctx, Some(line_height));
            }
        }
    } else {
        layout_text_segment(
            ctx,
            text_ctx,
            float_ctx,
            &Text {
                content: &text.content,
                node_id: text.id,
                style: text.style,
                text_desc: &text_desc,
            },
            line,
        );
    }
}

/// Measure a single-line text segment (no embedded newlines) and add it to
/// the current [`LineBox`], word-wrapping across multiple lines when the
/// text exceeds `available_width`.
fn layout_text_segment<'node>(
    ctx: &mut InlineLayoutContext<'node>,
    text_ctx: &mut TextContext,
    float_ctx: &FloatContext,
    text: &Text,
    line: &mut LineBoxBuilder<'node>,
) {
    let mut remaining_text = text.content;

    while !remaining_text.is_empty() {
        let available_width = line
            .line_box
            .available_width(float_ctx, ctx.available_width);
        let remaining_line_space = (available_width - line.line_box.width).max(0.0);

        if remaining_line_space < 1.0 && line.line_box.width > 0.0 {
            line.finish_line_with_decorations(ctx, text_ctx, float_ctx, None);
            continue;
        }

        let (measured, rest) = text_ctx.measure_text_that_fits(remaining_text, text.text_desc, remaining_line_space);

        if measured.width == 0.0 && measured.height == 0.0 {
            if let Some(r) = rest {
                remaining_text = r;
                continue;
            }
            break;
        }

        let node = LayoutNode::builder(text.node_id)
            .dimensions(Rect::new(0.0, 0.0, measured.width, measured.height))
            .colors(LayoutColors::text_only(text.style.color))
            .cursor(text.style.cursor)
            .text_buffer(Arc::new(measured.buffer))
            .height_auto(true)
            .build();

        let ascent = measured.height;
        let descent = 0.0;

        line.line_box.add(node, ascent, descent);

        if let Some(r) = rest {
            line.finish_line_with_decorations(ctx, text_ctx, float_ctx, None);
            remaining_text = r;
        } else {
            break;
        }
    }
}
