use crate::{ImageContext, LayoutEngine, LayoutNode, Rect, TextContext, float::FloatContext, layout::LayoutContext};
use css_style::StyledNode;

#[derive(Debug, Clone)]
struct PendingPosition {
    styled_node: Box<StyledNode>,
    containing_block: Rect,
}

#[derive(Debug, Clone, Default)]
pub struct PositionContext {
    pending: Vec<PendingPosition>,
    viewport: Rect,
    positioned: Vec<Rect>,
}

impl PositionContext {
    pub fn new(viewport: Rect) -> Self {
        Self {
            pending: Vec::new(),
            viewport,
            positioned: vec![viewport],
        }
    }

    pub fn push_position(&mut self, rect: Rect) {
        self.positioned.push(rect);
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

    pub fn defer(&mut self, styled_node: StyledNode, containing_block: Rect) {
        self.pending.push(PendingPosition {
            styled_node: Box::new(styled_node),
            containing_block,
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
                let mut ctx = LayoutContext::new(pending.containing_block);
                ctx.deferred = false;
                ctx.block_cursor.y = 0.0;
                ctx.set_positioned_containing_block(self.viewport);

                let node = LayoutEngine::layout_node(
                    &pending.styled_node,
                    &mut ctx,
                    &mut PositionContext::new(self.viewport),
                    float_ctx,
                    text_ctx,
                    image_ctx,
                )?;

                Some(node)
            })
            .collect()
    }
}
