use crate::{
    LayoutNode, LayoutTree, Rect,
    context::{LayoutContext, layout::Cursor},
    engine::LayoutInput,
    mode::block::{BlockContext, BlockLayout},
    primitives::Size,
};
use css_display::LayoutNodeId;
use css_style::ComputedStyle;

#[derive(Debug, Clone)]
struct PendingPosition {
    parent_id: LayoutNodeId,
    layout_id: LayoutNodeId,
    containing_block: Rect,
    block_ctx: BlockContext,
    defer_top_margin: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PositionContext {
    pending: Vec<PendingPosition>,
    viewport: Rect,
    positioned: Vec<Rect>,
    parent_id: LayoutNodeId,
}

impl PositionContext {
    pub fn new(viewport: Rect) -> Self {
        Self {
            pending: Vec::new(),
            viewport,
            positioned: vec![viewport],
            parent_id: LayoutNodeId::new(0),
        }
    }

    pub fn push_parent(&mut self, parent_id: &LayoutNodeId, rect: Rect) {
        self.parent_id = *parent_id;
        self.positioned.push(rect);
    }

    pub fn defer(
        &mut self,
        layout_id: &LayoutNodeId,
        containing_block: Option<Rect>,
        block_ctx: BlockContext,
        defer_top_margin: bool,
    ) {
        let containing_block = containing_block.unwrap_or(self.positioned.last().cloned().unwrap_or(self.viewport));
        self.pending.push(PendingPosition {
            parent_id: self.parent_id,
            layout_id: *layout_id,
            containing_block,
            block_ctx,
            defer_top_margin,
        });
    }

    pub fn _resolve_all(&mut self, input: &mut LayoutInput) -> Vec<(LayoutNode, Size)> {
        self.pending
            .drain(..)
            .filter_map(|mut pending| {
                let mut new_position_ctx = PositionContext::new(pending.containing_block);
                let mut ctx =
                    LayoutContext::deferred(Cursor { x: 0.0, y: 0.0 }, pending.containing_block, self.viewport);

                BlockLayout::layout(
                    &pending.layout_id,
                    &ComputedStyle::default(),
                    input,
                    &mut ctx,
                    &mut new_position_ctx,
                    &mut pending.block_ctx,
                    pending.defer_top_margin,
                )
            })
            .collect()
    }

    pub fn resolve_all(&mut self, input: &mut LayoutInput, layout_tree: &mut LayoutTree) {
        for mut pending in self.pending.drain(..) {
            let Some(layout_node) = layout_tree.find_node_by_layout_id(pending.parent_id) else {
                continue;
            };

            let mut new_position_ctx = PositionContext::new(pending.containing_block);
            let mut ctx = LayoutContext::deferred(
                Cursor { x: 0.0, y: 0.0 },
                Rect {
                    x: layout_node.dimensions.x,
                    y: layout_node.dimensions.y,
                    width: pending.containing_block.width,
                    height: pending.containing_block.height,
                },
                self.viewport,
            );

            if let Some((node, size)) = BlockLayout::layout(
                &pending.layout_id,
                &ComputedStyle::default(),
                input,
                &mut ctx,
                &mut new_position_ctx,
                &mut pending.block_ctx,
                pending.defer_top_margin,
            ) {
                layout_node.insert_child(node);
                layout_tree.content_width = layout_tree.content_width.max(size.width);
                layout_tree.content_height = layout_tree.content_height.max(size.height);
            }
        }
    }
}
