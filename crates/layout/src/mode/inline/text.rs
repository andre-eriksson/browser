use std::sync::Arc;

use css_display::LayoutNodeId;
use css_style::ComputedStyle;
use css_values::text::Whitespace;
use html_dom::NodeId;

use crate::{
    LayoutColors, LayoutNode, Rect, TextContext,
    context::{FloatContext, TextDescription},
    mode::inline::{InlineLayoutContext, collection::TextRun, line::LineBoxBuilder},
};

struct Text<'text> {
    content: &'text str,
    layout_id: LayoutNodeId,
    node_id: NodeId,
    style: &'text ComputedStyle,
    desc: &'text TextDescription<'text>,
}

pub fn layout_text<'node>(
    nodes: &mut Vec<Option<LayoutNode>>,
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
        let mut uses_source_layout_id = true;

        for (seg_idx, segment) in segments.iter().enumerate() {
            if !segment.is_empty() {
                let emitted = layout_text_segment(
                    nodes,
                    ctx,
                    text_ctx,
                    float_ctx,
                    &Text {
                        content: segment,
                        layout_id: *text.layout_id,
                        node_id: *text.node_id,
                        style: text.style,
                        desc: &text_desc,
                    },
                    line,
                    uses_source_layout_id,
                );

                if emitted {
                    uses_source_layout_id = false;
                }
            }

            if seg_idx < segments.len() - 1 {
                line.finish_line_with_decorations(nodes, ctx, text_ctx, float_ctx, Some(line_height));
            }
        }
    } else {
        layout_text_segment(
            nodes,
            ctx,
            text_ctx,
            float_ctx,
            &Text {
                content: &text.content,
                layout_id: *text.layout_id,
                node_id: *text.node_id,
                style: text.style,
                desc: &text_desc,
            },
            line,
            true,
        );
    }
}

/// Measure a single-line text segment (no embedded newlines) and add it to
/// the current [`LineBox`], word-wrapping across multiple lines when the
/// text exceeds `available_width`.
fn layout_text_segment<'node>(
    nodes: &mut Vec<Option<LayoutNode>>,
    ctx: &mut InlineLayoutContext<'node>,
    text_ctx: &mut TextContext,
    float_ctx: &FloatContext,
    text: &Text,
    line: &mut LineBoxBuilder<'node>,
    mut use_source_layout_id: bool,
) -> bool {
    let mut remaining_text = text.content;
    let mut emitted_any = false;

    while !remaining_text.is_empty() {
        let available_width = line
            .line_box
            .available_width(float_ctx, ctx.available_width);
        let remaining_line_space = (available_width - line.line_box.width).max(0.0);

        if remaining_line_space < 1.0 && line.line_box.width > 0.0 {
            line.finish_line_with_decorations(nodes, ctx, text_ctx, float_ctx, None);
            continue;
        }

        let (measured, rest) = text_ctx.measure_text_that_fits(remaining_text, text.desc, remaining_line_space);

        if measured.width == 0.0 && measured.height == 0.0 {
            if let Some(r) = rest {
                remaining_text = r;
                continue;
            }
            break;
        }

        let layout_id = if use_source_layout_id {
            use_source_layout_id = false;
            text.layout_id
        } else {
            let layout_id = LayoutNodeId::new(ctx.next_layout_id);
            ctx.next_layout_id += 1;
            layout_id
        };

        let mut node = LayoutNode::builder(layout_id)
            .dimensions(Rect::new(0.0, 0.0, measured.width, measured.height))
            .colors(LayoutColors::text_only(text.style.color))
            .cursor(text.style.cursor)
            .text_buffer(Arc::new(measured.buffer))
            .node_id(text.node_id)
            .build();

        let ascent = measured.height;
        let descent = 0.0;

        line.line_box.add(nodes, &mut node, ascent, descent);
        if layout_id.index() == nodes.len() {
            nodes.push(Some(node));
        } else {
            nodes[layout_id.index()] = Some(node);
        }
        ctx.ids.push(layout_id);
        emitted_any = true;

        if let Some(r) = rest {
            line.finish_line_with_decorations(nodes, ctx, text_ctx, float_ctx, None);
            remaining_text = r;
        } else {
            break;
        }
    }

    emitted_any
}
