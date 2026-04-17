use crate::{ImageContext, LayoutEngine, LayoutNode, Rect, TextContext, layout::LayoutContext};
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

    pub fn update_viewport(&mut self, viewport: Rect) {
        self.viewport = viewport;
    }

    pub fn push_position(&mut self, rect: Rect) {
        self.positioned.push(rect);
    }

    /// Returns the current number of positioned rects, for use with `offset_positions_since`.
    pub const fn position_count(&self) -> usize {
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

    pub fn resolve_all(&mut self, image_ctx: &ImageContext, text_ctx: &mut TextContext) -> Vec<LayoutNode> {
        self.pending
            .drain(..)
            .filter_map(|pending| {
                let mut new_position_ctx = PositionContext::new(pending.containing_block);
                let mut ctx =
                    LayoutContext::deferred(pending.containing_block, self.viewport, image_ctx, &mut new_position_ctx);

                LayoutEngine::layout_node(&pending.styled_node, &mut ctx, text_ctx)
            })
            .collect()
    }
}
