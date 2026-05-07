use crate::{ImageContext, LayoutEngine, LayoutNode, Rect, TextContext, layout::LayoutContext};
use css_style::StyleTree;
use html_dom::{DocumentRoot, NodeId};

#[derive(Debug, Clone)]
struct PendingPosition {
    node_id: NodeId,
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
    pub fn offset_positions_since(&mut self, start_count: usize, y_offset: f64) {
        for rect in self.positioned.iter_mut().skip(start_count) {
            rect.y += y_offset;
        }
    }

    pub fn defer(&mut self, node_id: &NodeId, containing_block: Rect) {
        self.pending.push(PendingPosition {
            node_id: *node_id,
            containing_block,
        });
    }

    pub fn resolve_all(
        &mut self,
        dom_tree: &DocumentRoot,
        style_tree: &StyleTree,
        image_ctx: &ImageContext,
        text_ctx: &mut TextContext,
    ) -> Vec<LayoutNode> {
        self.pending
            .drain(..)
            .filter_map(|pending| {
                let mut new_position_ctx = PositionContext::new(pending.containing_block);
                let mut ctx =
                    LayoutContext::deferred(pending.containing_block, self.viewport, image_ctx, &mut new_position_ctx);

                LayoutEngine::layout_node(dom_tree, style_tree, &pending.node_id, &mut ctx, text_ctx)
            })
            .collect()
    }
}
