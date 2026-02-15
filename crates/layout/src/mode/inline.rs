use std::sync::Arc;

use css_style::{
    ComputedStyle, StyledNode, TextAlign, Whitespace, WritingMode,
    display::{InsideDisplay, OutsideDisplay},
};
use html_dom::{HtmlTag, NodeId, Tag};

use crate::{
    LayoutColors, LayoutEngine, LayoutNode, Rect, SideOffset, TextContext, layout::LayoutContext,
    resolver::PropertyResolver, text::TextDescription,
};

/// Tracks an inline box decoration (background, border, padding) that needs to
/// be emitted as a `LayoutNode` once the line is finished and final positions
/// are known.
#[derive(Debug, Clone)]
struct InlineDecoration {
    id: NodeId,
    start_x: f32,
    end_x: f32,
    style: Box<ComputedStyle>,
    padding: SideOffset,
    border: SideOffset,
}

/// State for an inline box that has been opened but not yet closed.
#[derive(Debug, Clone)]
struct ActiveInlineBox {
    id: NodeId,
    style: Box<ComputedStyle>,
    start_x: f32,
    margin: SideOffset,
    padding: SideOffset,
    border: SideOffset,
}

/// A single line of inline layout, accumulating positioned `LayoutNode`s and
/// tracking the maximum ascent/descent for vertical alignment and any active
/// inline box decorations.
pub struct LineBox {
    items: Vec<LayoutNode>,
    width: f32,
    max_ascent: f32,
    max_descent: f32,
    x: f32,
    y: f32,
    decorations: Vec<InlineDecoration>,
}

impl LineBox {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            items: Vec::new(),
            width: 0.0,
            max_ascent: 0.0,
            max_descent: 0.0,
            x,
            y,
            decorations: Vec::new(),
        }
    }

    fn add(&mut self, mut node: LayoutNode, ascent: f32, descent: f32) {
        node.dimensions.x = self.x + self.width + node.resolved_margin.left;
        self.width += node.dimensions.width - node.resolved_margin.horizontal();

        self.max_ascent = self.max_ascent.max(ascent);
        self.max_descent = self.max_descent.max(descent);
        self.items.push(node);
    }

    /// Advance the line width by a fixed amount (e.g. for inline box
    /// padding/border contributions) without adding a layout node.
    fn advance(&mut self, amount: f32) {
        self.width += amount;
    }

    /// Finalise the line, emitting positioned `LayoutNode`s for any active
    /// inline box decorations and returning the nodes along with the total line height.
    fn finish(
        self,
        available_width: f32,
        text_align: &TextAlign,
        writing_mode: &WritingMode,
    ) -> (Vec<LayoutNode>, f32) {
        let line_height = self.max_ascent + self.max_descent;
        let mut final_nodes = Vec::new();

        let offset_x = match text_align {
            TextAlign::Left => 0.0,
            TextAlign::Center => (available_width - self.width) / 2.0,
            TextAlign::Right => available_width - self.width,
            TextAlign::Start => match writing_mode {
                WritingMode::HorizontalTb => 0.0,
                WritingMode::VerticalRl => available_width - self.width,
                WritingMode::VerticalLr => 0.0,
                WritingMode::SidewaysRl => 0.0,
                WritingMode::SidewaysLr => 0.0,
            },
            TextAlign::End => match writing_mode {
                WritingMode::HorizontalTb => available_width - self.width,
                WritingMode::VerticalRl => 0.0,
                WritingMode::VerticalLr => available_width - self.width,
                WritingMode::SidewaysRl => available_width - self.width,
                WritingMode::SidewaysLr => available_width - self.width,
            },
            TextAlign::Justify => 0.0, // TODO: implement justify by distributing extra space between words
            TextAlign::MatchParent => 0.0, // TODO: implement match-parent by inheriting text-align from parent
        };

        for dec in &self.decorations {
            let has_border = dec.border.top > 0.0
                || dec.border.right > 0.0
                || dec.border.bottom > 0.0
                || dec.border.left > 0.0;
            let has_background = dec.style.background_color.a > 0.0;

            if !has_border && !has_background {
                continue;
            }

            let dec_width = (dec.end_x - dec.start_x).max(0.0);
            let dec_height = line_height + dec.padding.vertical() + dec.border.vertical();

            let dec_x = self.x + dec.start_x + offset_x;
            let dec_y = self.y - dec.padding.top - dec.border.top;

            let mut node = LayoutNode::new(dec.id);
            node.dimensions = Rect::new(dec_x, dec_y, dec_width, dec_height);
            node.resolved_padding = dec.padding;
            node.resolved_border = dec.border;
            node.colors = LayoutColors::from(&dec.style);
            final_nodes.push(node);
        }

        for mut node in self.items {
            node.dimensions.x += offset_x;

            // TODO: vertical-align support.
            let baseline_y = self.y + self.max_ascent;

            node.dimensions.y = baseline_y - node.dimensions.height;

            final_nodes.push(node);
        }

        (final_nodes, line_height)
    }
}

/// An item in the intermediate representation of an inline layout, representing
/// either a run of text with a single style or the start/end of an inline box.
#[derive(Debug, Clone)]
pub enum InlineItem {
    /// A run of text with the same style
    TextRun {
        id: NodeId,
        text: String,
        style: Box<ComputedStyle>,
    },

    /// Marks the opening edge of an inline element (e.g. `<span>`).
    /// Contributes left border + left padding to the line and begins tracking
    /// a decoration region.
    InlineBoxStart {
        id: NodeId,
        style: Box<ComputedStyle>,
    },

    /// Marks the closing edge of an inline element.
    /// Contributes right border + right padding and finalises the decoration.
    InlineBoxEnd { id: NodeId },

    /// inline-block or inline flow-root
    InlineFlowRoot {
        node: Box<StyledNode>,
        style: Box<ComputedStyle>,
    },

    /// A line break, <br>
    Break { line_height_px: f32 },
}

pub struct InlineLayout;

impl InlineLayout {
    /// Collects inline items from the given styled nodes, recursively traversing into inline children but
    /// returning an error if it encounters a block-level element (which should be handled by the block layout instead).
    /// The resulting flat list of inline items is then canonicalised by collapsing whitespace
    /// according to the CSS `white-space` property of each text run and stripping leading/trailing whitespace from lines.
    pub fn collect_inline_items_from_nodes(
        parent_style: &ComputedStyle,
        nodes: &[StyledNode],
    ) -> Vec<InlineItem> {
        let mut raw_items = Vec::new();

        for node in nodes {
            let result = Self::collect(parent_style, node, &mut raw_items);

            if result.is_err() {
                break;
            }
        }

        Self::canonicalize_whitespace(raw_items)
    }

    /// Recursively collects inline items from the given styled node and its children,
    /// returning an error if it encounters a block-level element (which should be handled by the block layout instead).
    fn collect(
        style: &ComputedStyle,
        inline_node: &StyledNode,
        items: &mut Vec<InlineItem>,
    ) -> Result<(), ()> {
        if let Some(text) = inline_node.text_content.as_ref() {
            items.push(InlineItem::TextRun {
                id: inline_node.node_id,
                text: text.clone(),
                style: Box::new(style.inherited_subset()),
            });
        }

        if let Some(tag) = inline_node.tag.as_ref() {
            match tag {
                Tag::Html(HtmlTag::Br) => {
                    items.push(InlineItem::Break {
                        line_height_px: inline_node.style.line_height,
                    });
                }
                _ => {
                    let display = inline_node.style.display;

                    if display.outside() == Some(OutsideDisplay::Inline)
                        && display.inside() == Some(InsideDisplay::FlowRoot)
                    {
                        items.push(InlineItem::InlineFlowRoot {
                            node: inline_node.clone().into(),
                            style: Box::new(inline_node.style.clone()),
                        });

                        return Ok(());
                    } else if display.outside() != Some(OutsideDisplay::Inline) {
                        return Err(());
                    }

                    items.push(InlineItem::InlineBoxStart {
                        id: inline_node.node_id,
                        style: Box::new(inline_node.style.clone()),
                    });

                    for child in &inline_node.children {
                        Self::collect(&inline_node.style, child, items)?;
                    }

                    items.push(InlineItem::InlineBoxEnd {
                        id: inline_node.node_id,
                    });
                }
            }
        }

        Ok(())
    }

    /// The main entry point for inline layout: given a list of styled nodes that
    /// contribute to an inline formatting context, first collect them into a
    /// flat list of `InlineItem`s, then measure and position those items into
    /// one or more `LineBox`es according to the available width and text alignment,
    /// finally returning the positioned `LayoutNode`s and total height of the laid-out lines.
    pub fn layout(
        items: &[InlineItem],
        text_ctx: &mut TextContext,
        available_width: f32,
        start_x: f32,
        start_y: f32,
    ) -> (Vec<LayoutNode>, f32) {
        let mut nodes = Vec::new();
        let mut current_y = start_y;
        let mut line = LineBox::new(start_x, start_y);

        let mut last_text_align = TextAlign::Left;
        let mut last_writing_mode = WritingMode::HorizontalTb;

        let mut inline_box_stack: Vec<ActiveInlineBox> = Vec::new();

        for item in items {
            match item {
                InlineItem::TextRun { id, text, style } => {
                    if text.is_empty() {
                        continue;
                    }

                    let font_size = style.font_size;
                    let whitespace = style.whitespace;
                    let text_align = style.text_align;
                    let line_height = &style.line_height;
                    let font_family = &style.font_family;
                    let font_weight = style.font_weight;
                    let writing_mode = &style.writing_mode;

                    last_text_align = text_align;
                    last_writing_mode = *writing_mode;

                    let preserves_newlines = matches!(
                        whitespace,
                        Whitespace::Pre | Whitespace::PreWrap | Whitespace::PreLine
                    );

                    let text_desc = TextDescription {
                        whitespace: &whitespace,
                        line_height: *line_height,
                        font_family,
                        font_weight,
                        font_size_px: font_size,
                    };

                    if preserves_newlines && text.contains('\n') {
                        let segments: Vec<&str> = text.split('\n').collect();

                        for (seg_idx, segment) in segments.iter().enumerate() {
                            if !segment.is_empty() {
                                Self::layout_text_segment(
                                    text_ctx,
                                    *id,
                                    segment,
                                    style,
                                    &text_desc,
                                    available_width,
                                    start_x,
                                    &mut line,
                                    &mut nodes,
                                    &mut current_y,
                                    &mut inline_box_stack,
                                );
                            }

                            if seg_idx < segments.len() - 1 {
                                Self::finish_line_with_decorations(
                                    &mut line,
                                    &mut inline_box_stack,
                                    available_width,
                                    &text_align,
                                    writing_mode,
                                    &mut nodes,
                                    &mut current_y,
                                    Some(*line_height),
                                    start_x,
                                );
                            }
                        }
                    } else {
                        Self::layout_text_segment(
                            text_ctx,
                            *id,
                            text.as_str(),
                            style,
                            &text_desc,
                            available_width,
                            start_x,
                            &mut line,
                            &mut nodes,
                            &mut current_y,
                            &mut inline_box_stack,
                        );
                    }
                }
                InlineItem::InlineBoxStart { id, style } => {
                    let (margin, padding, border) = PropertyResolver::resolve_box_model(style);

                    let left_edge = margin.left + border.left + padding.left;
                    line.advance(left_edge);

                    inline_box_stack.push(ActiveInlineBox {
                        id: *id,
                        style: style.clone(),
                        start_x: line.width - left_edge + margin.left,
                        margin,
                        padding,
                        border,
                    });

                    last_text_align = style.text_align;
                    last_writing_mode = style.writing_mode;
                }
                InlineItem::InlineBoxEnd { id } => {
                    if let Some(pos) = inline_box_stack.iter().rposition(|b| b.id == *id) {
                        let active = inline_box_stack.remove(pos);

                        let right_edge =
                            active.padding.right + active.border.right + active.margin.right;
                        line.advance(right_edge);

                        line.decorations.push(InlineDecoration {
                            id: active.id,
                            start_x: active.start_x,
                            end_x: line.width - active.margin.right,
                            style: active.style,
                            padding: active.padding,
                            border: active.border,
                        });
                    }
                }
                InlineItem::InlineFlowRoot { node, style } => {
                    let (margin, padding, border) = PropertyResolver::resolve_box_model(style);

                    let desired_width = style.intrinsic_width;

                    let mut block_ctx = LayoutContext::new(Rect::new(0.0, 0.0, desired_width, 0.0));
                    let child_node = LayoutEngine::layout_node(node, &mut block_ctx, text_ctx);

                    if let Some(mut layout_node) = child_node {
                        let total_width = layout_node.dimensions.width
                            + padding.horizontal()
                            + border.horizontal();

                        let alignment = &style.text_align;
                        let writing_mode = &style.writing_mode;
                        last_text_align = *alignment;
                        last_writing_mode = *writing_mode;

                        if line.width + total_width > available_width && line.width > 0.0 {
                            Self::finish_line_with_decorations(
                                &mut line,
                                &mut inline_box_stack,
                                available_width,
                                alignment,
                                writing_mode,
                                &mut nodes,
                                &mut current_y,
                                None,
                                start_x,
                            );
                        }

                        let ascent = layout_node.dimensions.height + margin.top + margin.bottom;

                        layout_node.resolved_margin = margin;

                        line.add(layout_node, ascent, 0.0);
                    }
                }
                InlineItem::Break { line_height_px } => {
                    Self::finish_line_with_decorations(
                        &mut line,
                        &mut inline_box_stack,
                        available_width,
                        &last_text_align,
                        &last_writing_mode,
                        &mut nodes,
                        &mut current_y,
                        Some(*line_height_px),
                        start_x,
                    );
                }
            }
        }

        Self::close_active_decorations(&mut line, &mut inline_box_stack);

        let (line_nodes, h) = line.finish(available_width, &last_text_align, &last_writing_mode);
        nodes.extend(line_nodes);
        let total_height = current_y + h - start_y;

        (nodes, total_height)
    }

    /// Finishes the current line, emitting decorations for any active inline
    /// boxes, then starts a fresh line and re-opens those inline boxes on it.
    #[allow(clippy::too_many_arguments)]
    fn finish_line_with_decorations(
        line: &mut LineBox,
        inline_box_stack: &mut Vec<ActiveInlineBox>,
        available_width: f32,
        text_align: &TextAlign,
        writing_mode: &WritingMode,
        nodes: &mut Vec<LayoutNode>,
        current_y: &mut f32,
        min_line_height: Option<f32>,
        start_x: f32,
    ) {
        let continuing_boxes: Vec<(
            NodeId,
            Box<ComputedStyle>,
            SideOffset,
            SideOffset,
            SideOffset,
        )> = inline_box_stack
            .iter()
            .map(|b| (b.id, b.style.clone(), b.margin, b.padding, b.border))
            .collect();

        Self::close_active_decorations(line, inline_box_stack);

        let old_line = std::mem::replace(line, LineBox::new(start_x, *current_y));
        let (line_nodes, h) = old_line.finish(available_width, text_align, writing_mode);
        nodes.extend(line_nodes);
        *current_y += if let Some(min_h) = min_line_height {
            h.max(min_h)
        } else {
            h
        };
        *line = LineBox::new(start_x, *current_y);

        for (id, style, margin, padding, border) in continuing_boxes {
            inline_box_stack.push(ActiveInlineBox {
                id,
                style,
                start_x: line.width,
                margin,
                padding,
                border,
            });
        }
    }

    /// Close all active inline boxes, recording their decorations on the
    /// current line box and clearing the stack.
    fn close_active_decorations(line: &mut LineBox, inline_box_stack: &mut Vec<ActiveInlineBox>) {
        while let Some(active) = inline_box_stack.pop() {
            let right_edge = active.padding.right + active.border.right + active.margin.right;
            line.advance(right_edge);

            line.decorations.push(InlineDecoration {
                id: active.id,
                start_x: active.start_x,
                end_x: line.width - active.margin.right,
                style: active.style,
                padding: active.padding,
                border: active.border,
            });
        }
    }

    /// Measure a single-line text segment (no embedded newlines) and add it to
    /// the current [`LineBox`], word-wrapping across multiple lines when the
    /// text exceeds `available_width`.
    #[allow(clippy::too_many_arguments)]
    fn layout_text_segment(
        text_ctx: &mut TextContext,
        id: NodeId,
        text: &str,
        style: &ComputedStyle,
        text_desc: &TextDescription,
        available_width: f32,
        start_x: f32,
        line: &mut LineBox,
        nodes: &mut Vec<LayoutNode>,
        current_y: &mut f32,
        inline_box_stack: &mut Vec<ActiveInlineBox>,
    ) {
        let mut remaining_text = text;

        while !remaining_text.is_empty() {
            let remaining_line_space = (available_width - line.width).max(0.0);

            let (measured, rest) =
                text_ctx.measure_text_that_fits(remaining_text, text_desc, remaining_line_space);

            if measured.width == 0.0 && measured.height == 0.0 {
                if let Some(r) = rest {
                    remaining_text = r;
                    continue;
                }
                break;
            }

            let node = LayoutNode {
                node_id: id,
                dimensions: Rect::new(0.0, 0.0, measured.width, measured.height),
                colors: LayoutColors::from(style),
                resolved_margin: SideOffset::zero(),
                resolved_padding: SideOffset::zero(),
                resolved_border: SideOffset::zero(),
                text_buffer: Some(Arc::new(measured.buffer)),
                children: vec![],
            };

            let ascent = measured.height;
            let descent = 0.0;

            line.add(node, ascent, descent);

            if let Some(r) = rest {
                Self::finish_line_with_decorations(
                    line,
                    inline_box_stack,
                    available_width,
                    &style.text_align,
                    &style.writing_mode,
                    nodes,
                    current_y,
                    None,
                    start_x,
                );
                remaining_text = r;
            } else {
                break;
            }
        }
    }

    /// Canonicalise whitespace in the collected inline items according to the CSS
    /// `white-space` property of each text run, collapsing runs of whitespace into a single
    /// space where appropriate and stripping leading/trailing whitespace from lines.
    fn canonicalize_whitespace(items: Vec<InlineItem>) -> Vec<InlineItem> {
        let mut result = Vec::new();
        let mut last_was_space = false;

        for item in items {
            match item {
                InlineItem::TextRun { id, text, style } => {
                    let whitespace_prop = &style.whitespace;

                    if matches!(whitespace_prop, Whitespace::Pre | Whitespace::PreWrap) {
                        result.push(InlineItem::TextRun { id, text, style });
                        last_was_space = false;
                    } else {
                        let mut new_text = String::new();

                        for c in text.chars() {
                            if c.is_whitespace() {
                                if matches!(whitespace_prop, Whitespace::PreLine) && c == '\n' {
                                    new_text.push('\n');
                                    last_was_space = false;
                                } else if !last_was_space {
                                    new_text.push(' ');
                                    last_was_space = true;
                                }
                            } else {
                                new_text.push(c);
                                last_was_space = false;
                            }
                        }

                        if !new_text.is_empty() {
                            result.push(InlineItem::TextRun {
                                id,
                                text: new_text,
                                style,
                            });
                        }
                    }
                }
                other => {
                    result.push(other);
                    last_was_space = false;
                }
            }
        }

        Self::strip_edge_whitespace(&mut result);

        result
    }

    /// Returns true if the given style's `white-space` property preserves
    /// spaces (i.e. is `pre` or `pre-wrap`).
    fn preserves_spaces(style: &ComputedStyle) -> bool {
        matches!(style.whitespace, Whitespace::Pre | Whitespace::PreWrap)
    }

    /// Strips leading and trailing whitespace from the line, removing text runs that are entirely
    /// whitespace and trimming text runs at the edges. Stops stripping once it encounters a
    /// text run with a style that preserves spaces.
    fn strip_edge_whitespace(items: &mut Vec<InlineItem>) {
        while let Some(InlineItem::TextRun { text, style, .. }) = items.first() {
            if Self::preserves_spaces(style) {
                break;
            }
            let trimmed = text.trim_start();
            if trimmed.is_empty() {
                items.remove(0);
            } else {
                let t = trimmed.to_string();
                if let InlineItem::TextRun { text, .. } = &mut items[0] {
                    *text = t;
                }
                break;
            }
        }

        while let Some(InlineItem::TextRun { text, style, .. }) = items.last() {
            if Self::preserves_spaces(style) {
                break;
            }
            let trimmed = text.trim_end();
            if trimmed.is_empty() {
                items.pop();
            } else {
                let t = trimmed.to_string();
                let len = items.len();
                if let InlineItem::TextRun { text, .. } = &mut items[len - 1] {
                    *text = t;
                }
                break;
            }
        }
    }
}
