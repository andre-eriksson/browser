use std::sync::Arc;

use cosmic_text::Buffer;
use css_display::LayoutNodeId;
use css_style::ComputedStyle;
use css_values::text::Whitespace;
use html_dom::NodeId;

use crate::{
    LayoutColors, LayoutNode, Rect, TextContext,
    context::{FloatContext, TextDescription, TextFragment},
    mode::inline::{InlineLayoutContext, collection::TextRun, line::LineBoxBuilder},
};

// NOTE: Temporary fixed character width
const CHARACTER_WIDTH: f64 = 10.0;

struct TextInput<'text> {
    content: &'text str,
    layout_id: LayoutNodeId,
    node_id: NodeId,
    style: &'text ComputedStyle,
    desc: &'text TextDescription<'text>,
}

pub fn layout_text<'node>(
    nodes: &mut [Option<LayoutNode>],
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
            let input = TextInput {
                content: segment,
                layout_id: *text.layout_id,
                node_id: *text.node_id,
                style: text.style,
                desc: &text_desc,
            };

            if segment.is_empty() {
                flush_newline_marker(nodes, line, &input);
            } else {
                layout_text_segment(nodes, ctx, text_ctx, float_ctx, &input, line);
            }

            if seg_idx < segments.len() - 1 {
                line.finish_line_with_decorations(nodes, ctx, text_ctx, float_ctx, Some(line_height));
            }
        }

        ctx.ids.push(*text.layout_id);
    } else {
        layout_text_segment(
            nodes,
            ctx,
            text_ctx,
            float_ctx,
            &TextInput {
                content: &text.content,
                layout_id: *text.layout_id,
                node_id: *text.node_id,
                style: text.style,
                desc: &text_desc,
            },
            line,
        );

        ctx.ids.push(*text.layout_id);
    }
}

/// Measure a single-line text segment (no embedded newlines) and add it to
/// the current [`LineBox`], word-wrapping across multiple lines when the
/// text exceeds `available_width`.
fn layout_text_segment<'node>(
    nodes: &mut [Option<LayoutNode>],
    ctx: &mut InlineLayoutContext<'node>,
    text_ctx: &mut TextContext,
    float_ctx: &FloatContext,
    text: &TextInput,
    line: &mut LineBoxBuilder<'node>,
) {
    let mut remaining_text = text.content;
    let mut current_fragment_buffers: Vec<Arc<Buffer>> = Vec::new();
    let mut current_fragment_w = 0.0_f64;
    let mut current_fragment_h = 0.0_f64;

    while !remaining_text.is_empty() {
        let available_width = line
            .line_box
            .available_width(float_ctx, ctx.available_width)
            - CHARACTER_WIDTH;
        let remaining_line_space = (available_width - line.line_box.width).max(0.0);

        if remaining_line_space < 1.0 && line.line_box.width > 0.0 {
            flush_fragment(nodes, line, text, &mut current_fragment_buffers, current_fragment_w, current_fragment_h);
            current_fragment_w = 0.0;
            current_fragment_h = 0.0;
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

        current_fragment_w += measured.width;
        current_fragment_h = current_fragment_h.max(measured.height);
        current_fragment_buffers.push(Arc::new(measured.buffer));

        if let Some(r) = rest {
            let content =
                if !matches!(text.style.whitespace, Whitespace::Pre | Whitespace::PreWrap | Whitespace::PreLine) {
                    r.trim()
                } else {
                    r
                };

            flush_fragment(nodes, line, text, &mut current_fragment_buffers, current_fragment_w, current_fragment_h);
            current_fragment_w = 0.0;
            current_fragment_h = 0.0;
            line.finish_line_with_decorations(nodes, ctx, text_ctx, float_ctx, None);
            remaining_text = content;
        } else {
            break;
        }
    }

    flush_fragment(nodes, line, text, &mut current_fragment_buffers, current_fragment_w, current_fragment_h);
}

fn flush_fragment<'node>(
    nodes: &mut [Option<LayoutNode>],
    line: &mut LineBoxBuilder<'node>,
    text: &TextInput,
    buffers: &mut Vec<Arc<Buffer>>,
    width: f64,
    height: f64,
) {
    if buffers.is_empty() {
        return;
    }

    let node = nodes[text.layout_id.index()].get_or_insert_with(|| {
        LayoutNode::builder(text.layout_id)
            .colors(LayoutColors::text_only(text.style.color))
            .cursor(text.style.cursor)
            .node_id(text.node_id)
            .build()
    });

    let fragment = TextFragment {
        size: Rect::new(0.0, 0.0, width, height),
        buffers: std::mem::take(buffers),

        #[cfg(debug_assertions)]
        debug_content: text.content.to_string(),
    };
    let idx = node.text_fragments.len();
    node.text_fragments.push(fragment);
    node.dimensions.width += width;
    node.dimensions.height = node.dimensions.height.max(height);

    line.line_box
        .add_fragment(text.layout_id, idx, text.style, &mut node.text_fragments[idx].size, height, 0.0);
}

fn flush_newline_marker<'node>(nodes: &mut [Option<LayoutNode>], line: &mut LineBoxBuilder<'node>, text: &TextInput) {
    let line_height = text.style.line_height * text.style.font_size;
    let node = nodes[text.layout_id.index()].get_or_insert_with(|| {
        LayoutNode::builder(text.layout_id)
            .colors(LayoutColors::text_only(text.style.color))
            .cursor(text.style.cursor)
            .node_id(text.node_id)
            .build()
    });

    let fragment = TextFragment {
        size: Rect::new(0.0, 0.0, 0.0, line_height),
        buffers: Vec::new(),

        #[cfg(debug_assertions)]
        debug_content: String::new(),
    };
    let idx = node.text_fragments.len();
    node.text_fragments.push(fragment);

    line.line_box
        .add_fragment(text.layout_id, idx, text.style, &mut node.text_fragments[idx].size, line_height, 0.0);
}
