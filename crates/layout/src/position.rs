use std::collections::VecDeque;

use css_style::StyledNode;

use crate::{ImageContext, LayoutEngine, LayoutNode, Rect, TextContext, float::FloatContext, layout::LayoutContext};

#[derive(Debug, Clone)]
struct PendingPosition {
    styled_node: Box<StyledNode>,
}

#[derive(Debug, Clone, Default)]
pub struct PositionContext {
    pending: Vec<PendingPosition>,
    positioned: VecDeque<Rect>,
}

impl PositionContext {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            positioned: VecDeque::new(),
        }
    }

    pub fn push_position(&mut self, rect: Rect) {
        self.positioned.push_back(rect);
    }

    /// Returns the current number of positioned rects, for use with `offset_positions_since`.
    pub fn position_count(&self) -> usize {
        self.positioned.len()
    }

    /// Offsets the Y coordinate of all positioned rects added since `start_count`.
    /// This is used to apply margin offsets after layout completes for a subtree.
    pub fn offset_positions_since(&mut self, start_count: usize, y_offset: f32) {
        for rect in self.positioned.iter_mut().skip(start_count) {
            rect.y += y_offset;
        }
    }

    pub fn defer(&mut self, styled_node: StyledNode) {
        self.pending.push(PendingPosition {
            styled_node: Box::new(styled_node),
        });
    }

    pub fn resolve_all(
        &mut self,
        float_ctx: &mut FloatContext,
        text_ctx: &mut TextContext,
        image_ctx: &ImageContext,
    ) -> Vec<LayoutNode> {
        self.pending
            .drain(..)
            .filter_map(|pending| {
                let containing_block = self.positioned.pop_front().unwrap_or_default();
                let mut ctx = LayoutContext::new(containing_block);
                ctx.bypass = true;
                ctx.block_cursor.y = 0.0;

                LayoutEngine::layout_node(
                    &pending.styled_node,
                    &mut ctx,
                    &mut PositionContext::new(),
                    float_ctx,
                    text_ctx,
                    image_ctx,
                )
            })
            .collect()
    }
}
