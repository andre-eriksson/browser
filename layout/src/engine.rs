use css_style::{
    StyleTree, StyledNode,
    types::{
        display::{BoxDisplay, InsideDisplay},
        height::Height,
        margin::MarginValue,
        padding::PaddingValue,
        width::Width,
    },
};

use crate::{
    layout::{LayoutColors, LayoutNode, LayoutTree},
    primitives::{Color4f, Rect, SideOffset},
    text::TextContext,
};

/// Context passed down during layout computation
#[derive(Debug, Clone, Default)]
struct LayoutContext {
    /// The containing block's content rect (where children are positioned)
    pub containing_block: Rect,
}

/// Layout mode determines how children are positioned
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayoutMode {
    /// Default block layout
    Block,

    /// Flexbox layout
    Flex, // TODO: implement

    /// Grid layout
    Grid, // TODO: implement
}

impl LayoutMode {
    pub fn from_styled_node(styled_node: &StyledNode) -> Self {
        match styled_node.style.display.inside {
            Some(InsideDisplay::Flex) => LayoutMode::Flex,
            Some(InsideDisplay::Grid) => LayoutMode::Grid,
            _ => LayoutMode::Block,
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

        let root_nodes = style_tree
            .root_nodes
            .iter()
            .map(|styled_node| {
                let node = Self::layout_node(styled_node, &ctx, total_height, text_ctx);
                total_height += node.margin_box_height();
                node
            })
            .collect();

        LayoutTree {
            root_nodes,
            content_height: total_height,
        }
    }

    /// Compute layout for a single node and its descendants
    fn layout_node(
        styled_node: &StyledNode,
        ctx: &LayoutContext,
        flow_y: f32,
        text_ctx: &mut TextContext,
    ) -> LayoutNode {
        let layout_mode = LayoutMode::from_styled_node(styled_node);

        match layout_mode {
            LayoutMode::Block => Self::layout_block(styled_node, ctx, flow_y, text_ctx),
            LayoutMode::Flex => Self::layout_block(styled_node, ctx, flow_y, text_ctx), // TODO: implement flex layout
            LayoutMode::Grid => Self::layout_block(styled_node, ctx, flow_y, text_ctx), // TODO: implement grid layout
        }
    }

    /// Block layout: elements stack vertically
    fn layout_block(
        styled_node: &StyledNode,
        ctx: &LayoutContext,
        flow_y: f32,
        text_ctx: &mut TextContext,
    ) -> LayoutNode {
        if styled_node.style.display.box_display == Some(BoxDisplay::None) {
            return LayoutNode {
                node_id: styled_node.node_id,
                dimensions: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                },
                colors: LayoutColors::default(),
                resolved_margin: SideOffset::zero(),
                resolved_padding: SideOffset::zero(),
                children: vec![],
            };
        }

        let font_size_px = styled_node.style.computed_font_size_px;

        let margin = Self::resolve_margins(styled_node, ctx.containing_block.width);
        let padding = Self::resolve_padding(styled_node, ctx.containing_block.width);

        let colors = LayoutColors {
            background_color: Color4f::from_css_color(&styled_node.style.background_color),
            color: Color4f::from_css_color(&styled_node.style.color),
        };

        let x = ctx.containing_block.x + margin.left + padding.left;
        let y = ctx.containing_block.y + flow_y + margin.top + padding.top;

        let content_width = Self::calculate_width(styled_node, ctx, &margin, &padding);

        let child_ctx = LayoutContext {
            containing_block: Rect {
                x,
                y,
                width: content_width,
                height: ctx.containing_block.height,
            },
        };

        let (content_height, children) = if let Some(text) = &styled_node.text_content {
            let (_, text_height) = text_ctx.measure_text(
                text,
                font_size_px,
                &styled_node.style.line_height,
                &styled_node.style.font_family,
                content_width,
            );
            (text_height, vec![])
        } else {
            let mut child_flow_y = 0.0;
            let children: Vec<LayoutNode> = styled_node
                .children
                .iter()
                .map(|child| {
                    let child_node = Self::layout_node(child, &child_ctx, child_flow_y, text_ctx);
                    child_flow_y += child_node.margin_box_height();
                    child_node
                })
                .collect();

            let content_height = Self::calculate_height(styled_node, ctx, child_flow_y);
            (content_height, children)
        };

        let dimensions = Rect {
            x,
            y,
            width: content_width,
            height: content_height,
        };

        LayoutNode {
            node_id: styled_node.node_id,
            dimensions,
            colors,
            resolved_margin: margin,
            resolved_padding: padding,
            children,
        }
    }

    /// Resolve margin values to pixels
    fn resolve_margins(styled_node: &StyledNode, containing_width: f32) -> SideOffset {
        let margin = &styled_node.style.margin;
        SideOffset {
            top: Self::resolve_margin_value(&margin.top, containing_width),
            right: Self::resolve_margin_value(&margin.right, containing_width),
            bottom: Self::resolve_margin_value(&margin.bottom, containing_width),
            left: Self::resolve_margin_value(&margin.left, containing_width),
        }
    }

    /// Resolve a single margin value to pixels
    fn resolve_margin_value(value: &MarginValue, containing_width: f32) -> f32 {
        match value {
            MarginValue::Length(len) => len.value,
            MarginValue::Percentage(pct) => pct * containing_width / 100.0,
            MarginValue::Auto => 0.0,
            MarginValue::Global(_) => 0.0,
        }
    }

    /// Resolve padding values to pixels
    fn resolve_padding(styled_node: &StyledNode, containing_width: f32) -> SideOffset {
        let padding = &styled_node.style.padding;
        SideOffset {
            top: Self::resolve_padding_value(&padding.top, containing_width),
            right: Self::resolve_padding_value(&padding.right, containing_width),
            bottom: Self::resolve_padding_value(&padding.bottom, containing_width),
            left: Self::resolve_padding_value(&padding.left, containing_width),
        }
    }

    fn resolve_padding_value(value: &PaddingValue, containing_width: f32) -> f32 {
        match value {
            PaddingValue::Length(len) => len.value,
            PaddingValue::Percentage(pct) => pct * containing_width / 100.0,
            PaddingValue::Auto => 0.0,
            PaddingValue::Global(_) => 0.0,
        }
    }

    /// Calculate content width (top-down from containing block)
    fn calculate_width(
        styled_node: &StyledNode,
        ctx: &LayoutContext,
        margin: &SideOffset,
        padding: &SideOffset,
    ) -> f32 {
        let available_width =
            ctx.containing_block.width - margin.horizontal() - padding.horizontal();

        match &styled_node.style.width {
            Width::Auto => available_width.max(0.0),
            Width::Length(len) => len.value,
            Width::Percentage(pct) => (pct * ctx.containing_block.width / 100.0).max(0.0),
            Width::Global(_) => available_width.max(0.0),
            Width::MaxContent | Width::MinContent | Width::FitContent(_) | Width::Stretch => {
                // TODO: implement intrinsic sizing
                available_width.max(0.0)
            }
        }
    }

    /// Calculate content height (from explicit value or children)
    fn calculate_height(
        styled_node: &StyledNode,
        ctx: &LayoutContext,
        children_height: f32,
    ) -> f32 {
        match &styled_node.style.height {
            Height::Auto => children_height,
            Height::Length(len) => len.value,
            Height::Percentage(pct) => pct * ctx.containing_block.height / 100.0,
            Height::Global(_) => children_height,
            Height::MaxContent | Height::MinContent | Height::FitContent(_) | Height::Stretch => {
                // TODO: implement intrinsic sizing
                children_height
            }
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
    use html_syntax::dom::NodeId;

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
