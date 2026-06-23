use std::{collections::HashMap, f64};

use css_display::LayoutNodeId;
use css_style::ComputedStyle;
use css_values::text::{TextAlign, WritingMode};
use html_dom::NodeId;

use crate::{
    LayoutColors, LayoutNode, Rect, TextContext,
    context::{FloatContext, Geometry},
    mode::inline::{ActiveInlineBox, InlineDecoration, InlineLayoutContext},
};

/// A single line of inline layout, accumulating positioned `LayoutNode`s and
/// tracking the maximum ascent/descent for vertical alignment and any active
/// inline box decorations.
pub struct LineBox<'node> {
    pub items: Vec<LayoutNodeId>,
    pub fragments: HashMap<LayoutNodeId, Vec<(usize, Rect)>>,
    pub width: f64,
    pub max_ascent: f64,
    pub max_descent: f64,
    pub x: f64,
    pub y: f64,
    pub decorations: Vec<InlineDecoration<'node>>,
}

impl LineBox<'_> {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            items: Vec::with_capacity(8),
            fragments: HashMap::with_capacity(4),
            width: 0.0,
            max_ascent: 0.0,
            max_descent: 0.0,
            x,
            y,
            decorations: Vec::with_capacity(4),
        }
    }

    /// Get the available width for this line, accounting for floats
    pub fn available_width(&self, float_ctx: &FloatContext, container_width: f64) -> f64 {
        let (left_edge, right_edge) = float_ctx.available_width_at(self.y, container_width);
        (right_edge - left_edge).max(0.0)
    }

    pub fn add_fragment(
        &mut self,
        layout_id: LayoutNodeId,
        fragment_idx: usize,
        style: &ComputedStyle,
        size: &mut Rect,
        ascent: f64,
        descent: f64,
    ) {
        let margin_left = style.margin_left.to_px(self.width);
        let margin_right = style.margin_right.to_px(self.width);

        let new_x = self.x + self.width + margin_left;
        size.x = new_x;

        self.width += size.width + margin_left + margin_right;
        self.max_ascent = self.max_ascent.max(ascent);
        self.max_descent = self.max_descent.max(descent);
        self.fragments
            .entry(layout_id)
            .or_default()
            .push((fragment_idx, *size));
    }

    pub fn add_ascent(&mut self, ascent: f64) {
        self.max_ascent = self.max_ascent.max(ascent);
    }

    // pub fn add(&mut self, nodes: &mut Vec<Option<LayoutNode>>, node: &mut LayoutNode, ascent: f64, descent: f64) {
    //     let new_x = self.x + self.width + node.margin.left.to_px();
    //     let delta_x = new_x - node.dimensions.x;
    //     node.dimensions.x = new_x;

    //     if delta_x != 0.0 {
    //         LineBox::shift_descendants(nodes, &node.children, delta_x, 0.0);
    //     }

    //     self.width += node.dimensions.width + node.margin.left.to_px() + node.margin.right.to_px();

    //     self.max_ascent = self.max_ascent.max(ascent);
    //     self.max_descent = self.max_descent.max(descent);
    //     self.items.push(node.layout_id);
    // }

    // fn shift_descendants(nodes: &mut Vec<Option<LayoutNode>>, children: &[LayoutNodeId], delta_x: f64, delta_y: f64) {
    //     for child_id in children {
    //         let Some(mut node) = std::mem::take(&mut nodes[child_id.index()]) else {
    //             continue;
    //         };

    //         node.dimensions.x += delta_x;
    //         node.dimensions.y += delta_y;
    //         LineBox::shift_descendants(nodes, &node.children, delta_x, delta_y);
    //         nodes[child_id.index()] = Some(node);
    //     }
    // }

    /// Advance the line width by a fixed amount (e.g. for inline box
    /// padding/border contributions) without adding a layout node.
    pub fn advance(&mut self, amount: f64) {
        self.width += amount;
    }

    /// Finalise the line, emitting positioned `LayoutNode`s for any active
    /// inline box decorations and returning the nodes along with the total line height.
    pub fn finish(
        self,
        nodes: &mut [Option<LayoutNode>],
        float_ctx: &FloatContext,
        container_x: f64,
        container_width: f64,
        text_align: TextAlign,
        writing_mode: WritingMode,
    ) -> (Vec<LayoutNodeId>, f64) {
        let line_height = self.max_ascent + self.max_descent;
        let mut final_node_ids = Vec::with_capacity(self.decorations.len() + self.items.len());

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

            let node = LayoutNode::builder(dec.layout_id)
                .dimensions(Rect::new(dec_x, dec_y, dec_width, dec_height))
                .padding(dec.padding)
                .border(dec.border)
                .colors(LayoutColors::from(dec.style))
                .maybe_node_id(dec.node_id)
                .build();

            final_node_ids.push(dec.layout_id);
            nodes[dec.layout_id.index()] = Some(node);
        }

        for (id, sizes) in self.fragments.into_iter() {
            let Some(mut node) = std::mem::take(&mut nodes[id.index()]) else {
                continue;
            };

            for (idx, size) in sizes {
                // TODO: vertical-align support.
                let new_x = content_start_x + size.x - self.x + offset_x;
                let baseline_y = self.y + self.max_ascent;
                let new_y = baseline_y - size.height;

                node.text_fragments[idx].size = Rect::new(new_x, new_y, size.width, size.height);
            }

            final_node_ids.push(id);
            nodes[id.index()] = Some(node);
        }

        (final_node_ids, line_height)
    }
}

pub struct LineBoxBuilder<'node> {
    pub available_width: f64,
    pub line_box: LineBox<'node>,
}

impl<'node> LineBoxBuilder<'node> {
    pub fn new(available_width: f64, start_x: f64, start_y: f64) -> Self {
        Self {
            available_width,
            line_box: LineBox::new(start_x, start_y),
        }
    }

    pub(crate) fn open_inline_box(
        &mut self,
        inline_box_stack: &mut Vec<ActiveInlineBox<'node>>,
        text_ctx: &mut TextContext,
        layout_id: LayoutNodeId,
        node_id: Option<NodeId>,
        style: &'node ComputedStyle,
    ) {
        let box_model = Geometry::resolve_box_model(style, self.available_width);

        let left_edge = box_model.margin.left.to_px() + box_model.border.left + box_model.padding.left;
        self.line_box.advance(left_edge);

        inline_box_stack.push(ActiveInlineBox {
            layout_id,
            node_id,
            style,
            start_x: self.line_box.width - left_edge + box_model.margin.left.to_px(),
            margin: box_model.margin,
            padding: box_model.padding,
            border: box_model.border,
        });

        text_ctx.last_text_align = style.text_align;
        text_ctx.last_writing_mode = style.writing_mode;
    }

    pub(crate) fn close_inline_box(
        &mut self,
        inline_box_stack: &mut Vec<ActiveInlineBox<'node>>,
        layout_id: LayoutNodeId,
    ) {
        if let Some(pos) = inline_box_stack
            .iter()
            .rposition(|b| b.layout_id == layout_id)
        {
            let active = inline_box_stack.remove(pos);

            let right_edge = active.padding.right + active.border.right + active.margin.right.to_px();
            self.line_box.advance(right_edge);

            self.line_box.decorations.push(InlineDecoration {
                layout_id: active.layout_id,
                node_id: active.node_id,
                start_x: active.start_x,
                end_x: self.line_box.width - active.margin.right.to_px(),
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
        nodes: &mut [Option<LayoutNode>],
        ctx: &mut InlineLayoutContext<'node>,
        text_ctx: &TextContext,
        float_ctx: &FloatContext,
        min_line_height: Option<f64>,
    ) {
        let mut continuing_boxes = std::mem::take(&mut ctx.inline_box_stack);

        self.close_active_decorations(&mut continuing_boxes);

        let old_line = std::mem::replace(&mut self.line_box, LineBox::new(ctx.start_x, ctx.current_y));
        let (_, line_height) = old_line.finish(
            nodes,
            float_ctx,
            ctx.start_x,
            ctx.available_width,
            text_ctx.last_text_align,
            text_ctx.last_writing_mode,
        );
        ctx.current_y += min_line_height.map_or(line_height, |min_h| line_height.max(min_h));
        self.line_box = LineBox::new(ctx.start_x, ctx.current_y);

        for con_box in continuing_boxes {
            ctx.inline_box_stack.push(con_box);
        }
    }

    /// Close all active inline boxes, recording their decorations on the
    /// current line box and clearing the stack.
    pub(crate) fn close_active_decorations(&mut self, inline_box_stack: &mut Vec<ActiveInlineBox<'node>>) {
        while let Some(active) = inline_box_stack.pop() {
            let right_edge = active.padding.right + active.border.right + active.margin.right.to_px();
            self.line_box.advance(right_edge);

            self.line_box.decorations.push(InlineDecoration {
                layout_id: active.layout_id,
                node_id: active.node_id,
                start_x: active.start_x,
                end_x: self.line_box.width - active.margin.right.to_px(),
                style: active.style,
                padding: active.padding,
                border: active.border,
            });
        }
    }
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn add_accounts_for_horizontal_margins() {
    //     let mut line = LineBox::new(0.0, 0.0);
    //     let mut node = LayoutNode::builder(LayoutNodeId::new(0))
    //         .dimensions(Rect::new(0.0, 0.0, 10.0, 10.0))
    //         .margin(Margin {
    //             top: 0.0.into(),
    //             right: 3.0.into(),
    //             bottom: 0.0.into(),
    //             left: 2.0.into(),
    //         })
    //         .build();

    //     let mut nodes = vec![Some(node.clone())];
    //     line.add(&mut nodes, &mut node, 10.0, 0.0);
    //     let node = &nodes[0].clone().unwrap();

    //     assert_eq!(line.width, 15.0);
    //     assert_eq!(node.dimensions.x, 2.0);
    // }

    // #[test]
    // fn add_repositions_descendants_when_parent_x_changes() {
    //     let child = LayoutNode::builder(LayoutNodeId::new(1))
    //         .dimensions(Rect::new(5.0, 0.0, 4.0, 4.0))
    //         .build();
    //     let mut parent = LayoutNode::builder(LayoutNodeId::new(0))
    //         .dimensions(Rect::new(0.0, 0.0, 10.0, 10.0))
    //         .children(vec![LayoutNodeId::new(1)])
    //         .build();

    //     let mut nodes = vec![Some(parent.clone()), Some(child)];

    //     let mut line = LineBox::new(10.0, 0.0);
    //     line.add(&mut nodes, &mut parent, 10.0, 0.0);

    //     let first_item = &line.items[0];
    //     let first_node = &nodes[first_item.index()].clone().unwrap();
    //     assert_eq!(first_node.dimensions.x, 10.0);

    //     let first_child_id = &first_node.children[0];
    //     let first_child = &nodes[first_child_id.index()].clone().unwrap();

    //     assert_eq!(first_child.dimensions.x, 15.0);
    // }

    // #[test]
    // fn finish_repositions_descendants_with_parent() {
    //     let child = LayoutNode::builder(LayoutNodeId::new(1))
    //         .dimensions(Rect::new(2.0, 3.0, 4.0, 4.0))
    //         .build();
    //     let mut parent = LayoutNode::builder(LayoutNodeId::new(0))
    //         .dimensions(Rect::new(1.0, 2.0, 10.0, 10.0))
    //         .children(vec![LayoutNodeId::new(1)])
    //         .build();

    //     let mut nodes = vec![Some(parent.clone()), Some(child)];

    //     let mut line = LineBox::new(0.0, 40.0);
    //     line.add(&mut nodes, &mut parent, 10.0, 0.0);

    //     let _ = line.finish(&mut nodes, &FloatContext::new(), 10.0, 200.0, TextAlign::Left, WritingMode::HorizontalTb);

    //     let parent = &nodes[0].clone().unwrap();
    //     assert_eq!(parent.dimensions.x, 10.0);
    //     assert_eq!(parent.dimensions.y, 40.0);

    //     let child_id = &parent.children[0];
    //     let child = &nodes[child_id.index()].clone().unwrap();
    //     assert_eq!(child.dimensions.x, 11.0);
    //     assert_eq!(child.dimensions.y, 41.0);
    // }
}
