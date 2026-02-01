use css_style::{
    StyleTree, StyledNode,
    types::display::{BoxDisplay, InsideDisplay},
};

use crate::{
    layout::{LayoutContext, LayoutNode, LayoutTree},
    mode::block::{BlockCursor, BlockLayout},
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
        if styled_node.style.display.box_display == Some(BoxDisplay::None) {
            return None;
        }

        match styled_node.style.display.inside {
            Some(InsideDisplay::Flex) => Some(LayoutMode::Flex),
            Some(InsideDisplay::Grid) => Some(LayoutMode::Grid),
            _ => Some(LayoutMode::Block),
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
        let ctx = LayoutContext {
            containing_block: viewport,
            ..Default::default()
        };

        let mut total_height = 0.0;
        let mut block_cursor = BlockCursor { y: 0.0 };

        let mut root_nodes = Vec::new();

        for styled_node in &style_tree.root_nodes {
            let node = Self::layout_node(styled_node, &ctx, &mut block_cursor, text_ctx);

            if node.is_none() {
                // For `display: none`
                continue;
            }

            let node = node.unwrap();

            total_height += node.dimensions.height;
            root_nodes.push(node);
        }

        LayoutTree {
            root_nodes,
            content_height: total_height,
        }
    }

    /// Compute layout for a single node and its descendants
    pub(crate) fn layout_node(
        styled_node: &StyledNode,
        ctx: &LayoutContext,
        block_cursor: &mut BlockCursor,
        text_ctx: &mut TextContext,
    ) -> Option<LayoutNode> {
        let layout_mode = LayoutMode::new(styled_node)?;

        match layout_mode {
            LayoutMode::Block => Some(BlockLayout::layout(
                styled_node,
                ctx,
                block_cursor,
                text_ctx,
            )),
            LayoutMode::Flex => Some(BlockLayout::layout(
                styled_node,
                ctx,
                block_cursor,
                text_ctx,
            )), // TODO: implement flex layout
            LayoutMode::Grid => Some(BlockLayout::layout(
                styled_node,
                ctx,
                block_cursor,
                text_ctx,
            )), // TODO: implement grid layout
        }
    }
}

#[cfg(test)]
mod tests {
    use css_style::{
        ComputedStyle,
        types::{
            display::{Display, OutsideDisplay},
            height::Height,
            margin::{Margin, MarginValue},
            padding::{Padding, PaddingValue},
        },
    };
    use html_dom::{HtmlTag, NodeId, Tag};

    use super::*;

    fn viewport() -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        }
    }

    #[test]
    fn test_layout_mode_none() {
        let styled_node = StyledNode {
            style: ComputedStyle {
                display: Display {
                    box_display: Some(BoxDisplay::None),
                    ..Default::default()
                },
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
                display: Display {
                    outside: Some(OutsideDisplay::Block),
                    ..Default::default()
                },
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
                display: Display {
                    inside: Some(InsideDisplay::Flex),
                    ..Default::default()
                },
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
                display: Display {
                    inside: Some(InsideDisplay::Grid),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..StyledNode::new(NodeId(0))
        };
        assert_eq!(LayoutMode::new(&styled_node), Some(LayoutMode::Grid));
    }

    #[test]
    fn test_layout_example_1() {
        let node1 = StyledNode {
            style: ComputedStyle {
                height: Height::px(30.0),
                margin: Margin::all(MarginValue::px(20.0)),
                display: Display {
                    outside: Some(OutsideDisplay::Block),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..StyledNode::new(NodeId(2))
        };

        let node2 = StyledNode {
            style: ComputedStyle {
                height: Height::px(30.0),
                margin: Margin::all(MarginValue::px(20.0)),
                padding: Padding::all(PaddingValue::px(10.0)),
                display: Display {
                    outside: Some(OutsideDisplay::Block),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..StyledNode::new(NodeId(3))
        };

        let node3 = StyledNode {
            style: ComputedStyle {
                height: Height::px(30.0),
                margin: Margin::all(MarginValue::px(20.0)),
                display: Display {
                    outside: Some(OutsideDisplay::Block),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..StyledNode::new(NodeId(4))
        };

        let node4 = StyledNode {
            style: ComputedStyle {
                height: Height::px(30.0),
                margin: Margin::all(MarginValue::px(100.0)),
                display: Display {
                    outside: Some(OutsideDisplay::Block),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..StyledNode::new(NodeId(5))
        };

        let body = StyledNode {
            tag: Some(Tag::Html(HtmlTag::Body)),
            style: ComputedStyle {
                margin: Margin::all(MarginValue::px(8.0)),
                display: Display {
                    outside: Some(OutsideDisplay::Block),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![node1, node2, node3, node4],
            ..StyledNode::new(NodeId(1))
        };

        let html = StyledNode {
            tag: Some(Tag::Html(HtmlTag::Html)),
            style: ComputedStyle {
                display: Display {
                    outside: Some(OutsideDisplay::Block),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![body],
            ..StyledNode::new(NodeId(0))
        };

        let mut text_ctx = TextContext::default();
        let style_tree = StyleTree::from(html);

        let layout_tree = LayoutEngine::compute_layout(&style_tree, viewport(), &mut text_ctx);
        let body_layout = &layout_tree.root_nodes[0].children[0];

        assert_eq!(layout_tree.content_height, 400.0);
        assert_eq!(body_layout.dimensions.height, 280.0);
    }

    #[test]
    fn test_layout_example_1_with_padding() {
        let node1 = StyledNode {
            style: ComputedStyle {
                height: Height::px(30.0),
                margin: Margin::all(MarginValue::px(20.0)),
                display: Display {
                    outside: Some(OutsideDisplay::Block),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..StyledNode::new(NodeId(2))
        };

        let node2 = StyledNode {
            style: ComputedStyle {
                height: Height::px(30.0),
                margin: Margin::all(MarginValue::px(20.0)),
                padding: Padding::all(PaddingValue::px(10.0)),
                display: Display {
                    outside: Some(OutsideDisplay::Block),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..StyledNode::new(NodeId(3))
        };

        let node3 = StyledNode {
            style: ComputedStyle {
                height: Height::px(30.0),
                margin: Margin::all(MarginValue::px(20.0)),
                display: Display {
                    outside: Some(OutsideDisplay::Block),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..StyledNode::new(NodeId(4))
        };

        let node4 = StyledNode {
            style: ComputedStyle {
                height: Height::px(30.0),
                margin: Margin::all(MarginValue::px(100.0)),
                display: Display {
                    outside: Some(OutsideDisplay::Block),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..StyledNode::new(NodeId(5))
        };

        let body = StyledNode {
            tag: Some(Tag::Html(HtmlTag::Body)),
            style: ComputedStyle {
                padding: Padding::all(PaddingValue::px(10.0)),
                margin: Margin::all(MarginValue::px(8.0)),
                display: Display {
                    outside: Some(OutsideDisplay::Block),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![node1, node2, node3, node4],
            ..StyledNode::new(NodeId(1))
        };

        let html = StyledNode {
            tag: Some(Tag::Html(HtmlTag::Html)),
            style: ComputedStyle {
                display: Display {
                    outside: Some(OutsideDisplay::Block),
                    ..Default::default()
                },
                ..Default::default()
            },
            children: vec![body],
            ..StyledNode::new(NodeId(0))
        };

        let mut text_ctx = TextContext::default();
        let style_tree = StyleTree::from(html);

        let layout_tree = LayoutEngine::compute_layout(&style_tree, viewport(), &mut text_ctx);
        let body_layout = &layout_tree.root_nodes[0].children[0];

        assert_eq!(layout_tree.content_height, 436.0);
        assert_eq!(body_layout.dimensions.height, 420.0);
    }
}
