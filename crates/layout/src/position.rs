use css_style::{Position, StyledNode};
use html_dom::NodeId;

use crate::{ImageContext, LayoutEngine, LayoutNode, Rect, TextContext, layout::LayoutContext};

#[derive(Debug, Clone)]
#[allow(dead_code, reason = "TODO: Support all positions")]
struct PendingPosition {
    node_id: NodeId,
    strategy: Position,
    styled_node: Box<StyledNode>,
    containing_block: Rect,
}

#[derive(Debug, Clone, Default)]
pub struct PositionManager {
    pending: Vec<PendingPosition>,
}

impl PositionManager {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    #[allow(dead_code, reason = "TODO: Support positions")]
    pub fn defer(&mut self, node_id: NodeId, styled_node: StyledNode, containing_block: Rect) {
        self.pending.push(PendingPosition {
            node_id,
            strategy: styled_node.style.position,
            styled_node: Box::new(styled_node),
            containing_block,
        });
    }

    #[allow(dead_code, reason = "TODO: Support positions")]
    pub fn resolve_all(&mut self, text_ctx: &mut TextContext, image_ctx: &ImageContext) -> Vec<LayoutNode> {
        self.pending
            .drain(..)
            .filter_map(|pending| {
                let mut ctx = LayoutContext::new(pending.containing_block);
                LayoutEngine::layout_node(&pending.styled_node, &mut ctx, text_ctx, image_ctx)
            })
            .collect()
    }
}
