use css_style::ComputedStyle;
use css_values::text::{TextAlign, WritingMode};
use html_dom::NodeId;

use crate::{
    LayoutColors, LayoutNode, Rect, TextContext,
    float::FloatContext,
    mode::inline::{ActiveInlineBox, InlineDecoration, InlineLayoutContext},
    resolver::PropertyResolver,
};

/// A single line of inline layout, accumulating positioned `LayoutNode`s and
/// tracking the maximum ascent/descent for vertical alignment and any active
/// inline box decorations.
pub struct LineBox<'node> {
    pub items: Vec<LayoutNode>,
    pub width: f32,
    pub max_ascent: f32,
    pub max_descent: f32,
    pub x: f32,
    pub y: f32,
    pub decorations: Vec<InlineDecoration<'node>>,
}

impl<'node> LineBox<'node> {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            items: Vec::with_capacity(8),
            width: 0.0,
            max_ascent: 0.0,
            max_descent: 0.0,
            x,
            y,
            decorations: Vec::with_capacity(4),
        }
    }

    /// Get the available width for this line, accounting for floats
    pub fn available_width(&self, float_ctx: &FloatContext, container_width: f32) -> f32 {
        let (left_edge, right_edge) = float_ctx.available_width_at(self.y, container_width);
        (right_edge - left_edge).max(0.0)
    }

    pub fn add(&mut self, mut node: LayoutNode, ascent: f32, descent: f32) {
        node.dimensions.x = self.x + self.width + node.margin.left;
        self.width += node.dimensions.width - node.margin.horizontal();

        self.max_ascent = self.max_ascent.max(ascent);
        self.max_descent = self.max_descent.max(descent);
        self.items.push(node);
    }

    /// Advance the line width by a fixed amount (e.g. for inline box
    /// padding/border contributions) without adding a layout node.
    pub fn advance(&mut self, amount: f32) {
        self.width += amount;
    }

    /// Finalise the line, emitting positioned `LayoutNode`s for any active
    /// inline box decorations and returning the nodes along with the total line height.
    pub fn finish(
        self,
        float_ctx: &FloatContext,
        container_x: f32,
        container_width: f32,
        text_align: &TextAlign,
        writing_mode: &WritingMode,
    ) -> (Vec<LayoutNode>, f32) {
        let line_height = self.max_ascent + self.max_descent;
        let mut final_nodes = Vec::with_capacity(self.decorations.len() + self.items.len());

        let (left_edge, right_edge) = float_ctx.available_width_at(self.y, container_width);
        let available_width = (right_edge - left_edge).max(0.0);
        let content_start_x = container_x + left_edge;

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
            let has_border =
                dec.border.top > 0.0 || dec.border.right > 0.0 || dec.border.bottom > 0.0 || dec.border.left > 0.0;
            let has_background = dec.style.background_color.a > 0.0;

            if !has_border && !has_background {
                continue;
            }

            let dec_width = (dec.end_x - dec.start_x).max(0.0);
            let dec_height = line_height + dec.padding.vertical() + dec.border.vertical();

            let dec_x = content_start_x + dec.start_x + offset_x;
            let dec_y = self.y - dec.padding.top - dec.border.top;

            let node = LayoutNode::builder(dec.id)
                .dimensions(Rect::new(dec_x, dec_y, dec_width, dec_height))
                .padding(dec.padding)
                .border(dec.border)
                .colors(LayoutColors::from(dec.style))
                .build();

            final_nodes.push(node);
        }

        for mut node in self.items {
            node.dimensions.x = content_start_x + node.dimensions.x - self.x + offset_x;
            // TODO: vertical-align support.
            let baseline_y = self.y + self.max_ascent;
            node.dimensions.y = baseline_y - node.dimensions.height;

            final_nodes.push(node);
        }

        (final_nodes, line_height)
    }
}

pub struct LineBoxBuilder<'node> {
    pub line_box: LineBox<'node>,
}

impl<'node> LineBoxBuilder<'node> {
    pub fn new(start_x: f32, start_y: f32) -> Self {
        Self {
            line_box: LineBox::new(start_x, start_y),
        }
    }

    pub(crate) fn open_inline_box(
        &mut self,
        inline_box_stack: &mut Vec<ActiveInlineBox<'node>>,
        text_ctx: &mut TextContext,
        id: NodeId,
        style: &'node ComputedStyle,
    ) {
        let (margin, padding, border) = PropertyResolver::resolve_box_model(style);

        let left_edge = margin.left + border.left + padding.left;
        self.line_box.advance(left_edge);

        inline_box_stack.push(ActiveInlineBox {
            id,
            style,
            start_x: self.line_box.width - left_edge + margin.left,
            margin,
            padding,
            border,
        });

        text_ctx.last_text_align = style.text_align;
        text_ctx.last_writing_mode = style.writing_mode;
    }

    pub(crate) fn close_inline_box(&mut self, inline_box_stack: &mut Vec<ActiveInlineBox<'node>>, id: NodeId) {
        if let Some(pos) = inline_box_stack.iter().rposition(|b| b.id == id) {
            let active = inline_box_stack.remove(pos);

            let right_edge = active.padding.right + active.border.right + active.margin.right;
            self.line_box.advance(right_edge);

            self.line_box.decorations.push(InlineDecoration {
                id: active.id,
                start_x: active.start_x,
                end_x: self.line_box.width - active.margin.right,
                style: active.style,
                padding: active.padding,
                border: active.border,
            });
        }
    }

    /// Finishes the current line, emitting decorations for any active inline
    /// boxes, then starts a fresh line and re-opens those inline boxes on it.
    pub(crate) fn finish_line_with_decorations(
        &mut self,
        ctx: &mut InlineLayoutContext<'_, 'node>,
        text_ctx: &TextContext,
        min_line_height: Option<f32>,
    ) {
        let mut continuing_boxes = std::mem::take(ctx.inline_box_stack);

        self.close_active_decorations(&mut continuing_boxes);

        let old_line = std::mem::replace(&mut self.line_box, LineBox::new(ctx.start_x, *ctx.current_y));
        let (line_nodes, h) = old_line.finish(
            ctx.float_context,
            ctx.start_x,
            ctx.available_width,
            &text_ctx.last_text_align,
            &text_ctx.last_writing_mode,
        );
        ctx.nodes.extend(line_nodes);
        *ctx.current_y += min_line_height.map_or(h, |min_h| h.max(min_h));
        self.line_box = LineBox::new(ctx.start_x, *ctx.current_y);

        for con_box in continuing_boxes {
            ctx.inline_box_stack.push(con_box);
        }
    }

    /// Close all active inline boxes, recording their decorations on the
    /// current line box and clearing the stack.
    pub(crate) fn close_active_decorations(&mut self, inline_box_stack: &mut Vec<ActiveInlineBox<'node>>) {
        while let Some(active) = inline_box_stack.pop() {
            let right_edge = active.padding.right + active.border.right + active.margin.right;
            self.line_box.advance(right_edge);

            self.line_box.decorations.push(InlineDecoration {
                id: active.id,
                start_x: active.start_x,
                end_x: self.line_box.width - active.margin.right,
                style: active.style,
                padding: active.padding,
                border: active.border,
            });
        }
    }
}
