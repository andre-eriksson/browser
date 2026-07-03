use crate::{
    LayoutTree, Rect,
    context::{BoxModel, FloatContext, LayoutContext, layout::Cursor},
    engine::LayoutInput,
    mode::block::{BlockContext, BlockLayout},
};
use css_display::LayoutNodeId;
use css_style::ComputedStyle;

#[derive(Debug, Clone)]
struct PendingPosition {
    parent_id: LayoutNodeId,
    layout_id: LayoutNodeId,
    containing_block: Rect,
    block_ctx: BlockContext,
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

    pub fn defer(&mut self, layout_id: &LayoutNodeId, containing_block: Option<Rect>, block_ctx: BlockContext) {
        let containing_block = containing_block.unwrap_or(self.positioned.last().cloned().unwrap_or(self.viewport));
        self.pending.push(PendingPosition {
            parent_id: self.parent_id,
            layout_id: *layout_id,
            containing_block,
            block_ctx,
        });
    }

    pub fn resolve_all(&mut self, input: &mut LayoutInput, layout_tree: &mut LayoutTree) {
        for mut pending in self.pending.drain(..) {
            let Some(mut layout_node) = std::mem::take(&mut layout_tree.nodes[pending.parent_id.index()]) else {
                continue;
            };

            // TODO: Restore old contexts
            let mut new_position_ctx = PositionContext::new(pending.containing_block);
            let mut float_ctx = FloatContext::new();
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
                &mut layout_tree.nodes,
                &pending.layout_id,
                &ComputedStyle::default(),
                input,
                &mut ctx,
                &mut new_position_ctx,
                &mut pending.block_ctx,
                &mut float_ctx,
            ) {
                layout_node.insert_child(node);

                let box_model = BoxModel::from(&layout_node);
                let height = BlockLayout::calculate_height(
                    &input.box_tree[layout_node.layout_id].style,
                    &box_model,
                    size.height,
                    size.height,
                );

                layout_node.dimensions.height = layout_node.dimensions.height.max(height);
                layout_tree.content_width = layout_tree.content_width.max(size.width);
                layout_tree.content_height = layout_tree.content_height.max(size.height);
            }

            layout_tree.nodes[pending.parent_id.index()] = Some(layout_node);
        }
    }
}
