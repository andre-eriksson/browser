use css_style::{
    CSSProperty, StyleTree, StyledNode,
    display::{BoxDisplay, InsideDisplay},
};

use crate::{
    layout::{LayoutContext, LayoutNode, LayoutTree},
    mode::block::BlockLayout,
    primitives::Rect,
    text::TextContext,
};

/// Layout mode determines how children are positioned
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LayoutMode {
    Block,
    Flex, // TODO: implement
    Grid, // TODO: implement
}

impl LayoutMode {
    pub fn new(styled_node: &StyledNode) -> Option<Self> {
        if let Ok(display) = CSSProperty::resolve(&styled_node.style.display) {
            if display.box_display() == Some(BoxDisplay::None) {
                return None;
            }

            match display.inside() {
                Some(InsideDisplay::Flex) => Some(LayoutMode::Flex),
                Some(InsideDisplay::Grid) => Some(LayoutMode::Grid),
                _ => Some(LayoutMode::Block),
            }
        } else {
            Some(LayoutMode::Block)
        }
    }
}

pub struct LayoutEngine;

impl LayoutEngine {
    /// Main entry point: compute layout for an entire style tree
    pub fn compute_layout(
        style_tree: &StyleTree,
        viewport: Rect,
        text_ctx: &mut TextContext,
    ) -> LayoutTree {
        let mut ctx = LayoutContext::new(viewport);

        let mut total_height = 0.0;
        let mut root_nodes = Vec::new();

        for styled_node in &style_tree.root_nodes {
            ctx.block_cursor.y = total_height;

            let node = Self::layout_node(styled_node, &mut ctx, text_ctx);

            if node.is_none() {
                // For `display: none`
                continue;
            }

            let mut node = node.unwrap();

            let top_margin = node.resolved_margin.top;
            let bottom_margin = node.resolved_margin.bottom;

            Self::offset_children_y(&mut node.children, top_margin);

            node.dimensions.height += top_margin + bottom_margin;

            total_height += node.dimensions.height;
            root_nodes.push(node);
        }

        LayoutTree {
            root_nodes,
            content_height: total_height,
        }
    }

    /// Recursively offset all children's y positions
    fn offset_children_y(children: &mut [LayoutNode], offset: f32) {
        for child in children.iter_mut() {
            child.dimensions.y += offset;
            Self::offset_children_y(&mut child.children, offset);
        }
    }

    /// Compute layout for a single node and its descendants
    pub(crate) fn layout_node(
        styled_node: &StyledNode,
        ctx: &mut LayoutContext,
        text_ctx: &mut TextContext,
    ) -> Option<LayoutNode> {
        let layout_mode = LayoutMode::new(styled_node)?;

        match layout_mode {
            LayoutMode::Block => Some(BlockLayout::layout(styled_node, ctx, text_ctx)),
            LayoutMode::Flex => Some(BlockLayout::layout(styled_node, ctx, text_ctx)), // TODO: implement flex layout
            LayoutMode::Grid => Some(BlockLayout::layout(styled_node, ctx, text_ctx)), // TODO: implement grid layout
        }
    }
}

#[cfg(test)]
mod tests {
    use css_style::{ComputedStyle, Display, display::OutsideDisplay};
    use html_dom::NodeId;

    use crate::mode::block::BlockCursor;

    use super::*;

    fn viewport() -> Rect {
        Rect::new(0.0, 0.0, 800.0, 600.0)
    }

    #[test]
    fn test_layout_mode_none() {
        let styled_node = StyledNode {
            style: ComputedStyle {
                display: CSSProperty::from(Display::from(BoxDisplay::None)),
                ..Default::default()
            },
            ..StyledNode::new(NodeId(0))
        };

        assert_eq!(LayoutMode::new(&styled_node), None);
    }

    #[test]
    fn test_layout_mode_block() {
        let styled_node = StyledNode {
            style: ComputedStyle {
                display: CSSProperty::from(Display::from(OutsideDisplay::Block)),
                ..Default::default()
            },
            ..StyledNode::new(NodeId(0))
        };

        assert_eq!(LayoutMode::new(&styled_node), Some(LayoutMode::Block));
    }

    #[test]
    fn test_layout_mode_flex() {
        let styled_node = StyledNode {
            style: ComputedStyle {
                display: CSSProperty::from(Display::from(InsideDisplay::Flex)),
                ..Default::default()
            },
            ..StyledNode::new(NodeId(0))
        };
        assert_eq!(LayoutMode::new(&styled_node), Some(LayoutMode::Flex));
    }

    #[test]
    fn test_layout_mode_grid() {
        let styled_node = StyledNode {
            style: ComputedStyle {
                display: CSSProperty::from(Display::from(InsideDisplay::Grid)),
                ..Default::default()
            },
            ..StyledNode::new(NodeId(0))
        };
        assert_eq!(LayoutMode::new(&styled_node), Some(LayoutMode::Grid));
    }

    #[test]
    fn test_layout_empty() {
        let styled_node = StyledNode {
            style: ComputedStyle {
                display: CSSProperty::from(Display::from(OutsideDisplay::Block)),
                ..Default::default()
            },
            ..StyledNode::new(NodeId(0))
        };

        let mut ctx = LayoutContext::new(viewport());
        let cursor = BlockCursor { y: 0.0 };

        ctx.block_cursor = cursor;
        let mut text_ctx = TextContext::default();

        let layout_node = BlockLayout::layout(&styled_node, &mut ctx, &mut text_ctx);

        assert_eq!(layout_node.node_id, styled_node.node_id);
        assert_eq!(layout_node.dimensions.x, 0.0);
        assert_eq!(layout_node.dimensions.y, 0.0);
        assert_eq!(layout_node.dimensions.width, 800.0);
        assert_eq!(layout_node.dimensions.height, 0.0);
        assert_eq!(layout_node.children.len(), 0);
    }
}
