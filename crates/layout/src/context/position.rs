use crate::{
    LayoutNode, LayoutTree, Rect,
    context::{LayoutContext, layout::Cursor},
    engine::LayoutInput,
    mode::block::{BlockContext, BlockLayout},
    primitives::Size,
};
use css_display::BoxNode;
use css_style::ComputedStyle;

#[derive(Debug, Clone)]
struct PendingPosition<'node> {
    box_node: BoxNode<'node>,
    containing_block: Rect,
}

#[derive(Debug, Clone, Default)]
pub struct PositionContext<'node> {
    pending: Vec<PendingPosition<'node>>,
    viewport: Rect,
    positioned: Vec<Rect>,
}

impl<'node> PositionContext<'node> {
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
    pub fn offset_positions_since(&mut self, start_count: usize, y_offset: f64) {
        for rect in self.positioned.iter_mut().skip(start_count) {
            rect.y += y_offset;
        }
    }

    pub fn defer(&mut self, box_node: &'node BoxNode, containing_block: Rect) {
        self.pending.push(PendingPosition {
            box_node: box_node.clone(),
            containing_block,
        });
    }

    pub fn resolve_all(&mut self, input: &mut LayoutInput) -> Vec<(LayoutNode, Size)> {
        self.pending
            .drain(..)
            .filter_map(|pending| {
                let mut ctx =
                    LayoutContext::deferred(Cursor { x: 0.0, y: 0.0 }, pending.containing_block, self.viewport);
                let mut block_ctx = BlockContext::default();

                BlockLayout::layout(
                    &pending.box_node,
                    &ComputedStyle::default(),
                    input,
                    &mut ctx,
                    &mut block_ctx,
                    false,
                )
            })
            .collect()
    }
}
