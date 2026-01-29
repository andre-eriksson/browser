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
    pub fn from_styled_node(styled_node: &StyledNode) -> Option<Self> {
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

            total_height +=
                node.resolved_margin.bottom + node.resolved_padding.bottom + node.dimensions.height;
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
        let layout_mode = LayoutMode::from_styled_node(styled_node)?;

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
    use super::*;
    use css_style::ComputedStyle;
    use css_style::types::{
        height::Height,
        length::{Length, LengthUnit},
        margin::Margin,
        margin::MarginValue,
    };
    use html_dom::NodeId;

    fn viewport() -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        }
    }

    #[test]
    fn test_single_block_with_margin() {
        let style_tree = StyleTree {
            root_nodes: vec![StyledNode {
                node_id: NodeId(0),
                tag: None,
                style: ComputedStyle {
                    height: Height::Length(Length {
                        value: 100.0,
                        unit: LengthUnit::Px,
                    }),
                    margin: Margin::all(MarginValue::Length(Length {
                        value: 10.0,
                        unit: LengthUnit::Px,
                    })),
                    ..Default::default()
                },
                children: vec![],
                text_content: None,
            }],
        };

        let layout_tree =
            LayoutEngine::compute_layout(&style_tree, viewport(), &mut TextContext::default());

        assert_eq!(layout_tree.root_nodes.len(), 1);
        let layout_node = &layout_tree.root_nodes[0];

        assert_eq!(layout_node.dimensions.x, 10.0);
        assert_eq!(layout_node.dimensions.y, 10.0);
        assert_eq!(layout_node.dimensions.height, 100.0);
        assert_eq!(layout_node.dimensions.width, 780.0);
    }

    #[test]
    fn test_parent_with_child() {
        let style_node_child = StyledNode {
            node_id: NodeId(1),
            tag: None,
            style: ComputedStyle {
                height: Height::Length(Length {
                    value: 50.0,
                    unit: LengthUnit::Px,
                }),
                margin: Margin::all(MarginValue::Length(Length {
                    value: 5.0,
                    unit: LengthUnit::Px,
                })),
                ..Default::default()
            },
            children: vec![],
            text_content: None,
        };

        let style_node_parent = StyledNode {
            node_id: NodeId(0),
            tag: None,
            style: ComputedStyle {
                height: Height::Length(Length {
                    value: 100.0,
                    unit: LengthUnit::Px,
                }),
                margin: Margin::all(MarginValue::Length(Length {
                    value: 10.0,
                    unit: LengthUnit::Px,
                })),
                ..Default::default()
            },
            children: vec![style_node_child],
            text_content: None,
        };

        let style_tree = StyleTree {
            root_nodes: vec![style_node_parent],
        };

        let layout_tree =
            LayoutEngine::compute_layout(&style_tree, viewport(), &mut TextContext::default());

        assert_eq!(layout_tree.root_nodes.len(), 1);
        let parent = &layout_tree.root_nodes[0];

        assert_eq!(parent.dimensions.x, 10.0);
        assert_eq!(parent.dimensions.y, 10.0);

        let child = &parent.children[0];
        assert_eq!(child.dimensions.x, 15.0);
        assert_eq!(child.dimensions.y, 15.0);
        assert_eq!(child.dimensions.height, 50.0);
    }

    #[test]
    fn test_siblings_do_not_accumulate_x() {
        let sibling1 = StyledNode {
            node_id: NodeId(1),
            tag: None,
            style: ComputedStyle {
                height: Height::Length(Length {
                    value: 30.0,
                    unit: LengthUnit::Px,
                }),
                margin: Margin::all(MarginValue::Length(Length {
                    value: 20.0,
                    unit: LengthUnit::Px,
                })),
                ..Default::default()
            },
            children: vec![],
            text_content: None,
        };

        let sibling2 = StyledNode {
            node_id: NodeId(2),
            tag: None,
            style: ComputedStyle {
                height: Height::Length(Length {
                    value: 30.0,
                    unit: LengthUnit::Px,
                }),
                margin: Margin::all(MarginValue::Length(Length {
                    value: 20.0,
                    unit: LengthUnit::Px,
                })),
                ..Default::default()
            },
            children: vec![],
            text_content: None,
        };

        let parent = StyledNode {
            node_id: NodeId(0),
            tag: None,
            style: ComputedStyle {
                height: Height::Auto,
                margin: Margin::zero(),
                ..Default::default()
            },
            children: vec![sibling1, sibling2],
            text_content: None,
        };

        let style_tree = StyleTree {
            root_nodes: vec![parent],
        };

        let layout_tree =
            LayoutEngine::compute_layout(&style_tree, viewport(), &mut TextContext::default());

        let parent = &layout_tree.root_nodes[0];
        let child1 = &parent.children[0];
        let child2 = &parent.children[1];

        assert_eq!(child1.dimensions.x, 20.0);
        assert_eq!(child2.dimensions.x, 20.0);
        assert_eq!(child1.dimensions.y, 20.0);
        assert_eq!(child2.dimensions.y, 90.0);
    }

    #[test]
    fn test_auto_height_from_children() {
        let child = StyledNode {
            node_id: NodeId(1),
            tag: None,
            style: ComputedStyle {
                height: Height::Length(Length {
                    value: 50.0,
                    unit: LengthUnit::Px,
                }),
                margin: Margin::zero(),
                ..Default::default()
            },
            children: vec![],
            text_content: None,
        };

        let parent = StyledNode {
            node_id: NodeId(0),
            tag: None,
            style: ComputedStyle {
                height: Height::Auto,
                margin: Margin::zero(),
                ..Default::default()
            },
            children: vec![child],
            text_content: None,
        };

        let style_tree = StyleTree {
            root_nodes: vec![parent],
        };

        let layout_tree =
            LayoutEngine::compute_layout(&style_tree, viewport(), &mut TextContext::default());

        let parent = &layout_tree.root_nodes[0];
        assert_eq!(parent.dimensions.height, 50.0);
    }

    #[test]
    fn test_color_extraction() {
        use css_style::types::color::{Color, NamedColor};

        let styled_node = StyledNode {
            node_id: NodeId(0),
            tag: None,
            style: ComputedStyle {
                background_color: Color::Hex([255, 0, 0]),
                color: Color::Named(NamedColor::White),
                ..Default::default()
            },
            children: vec![],
            text_content: None,
        };

        let style_tree = StyleTree {
            root_nodes: vec![styled_node],
        };

        let layout_tree =
            LayoutEngine::compute_layout(&style_tree, viewport(), &mut TextContext::default());

        let colors = &layout_tree.root_nodes[0].colors;
        assert_eq!(colors.background_color.r, 1.0);
        assert_eq!(colors.background_color.g, 0.0);
        assert_eq!(colors.background_color.b, 0.0);
        assert_eq!(colors.background_color.a, 1.0);
    }
}
