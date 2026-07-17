use css_display::LayoutNodeId;
use css_style::{ComputedSize, ComputedStyle, Position};
use css_values::display::Float;
use tracing::{Level, enabled, trace};

use crate::{
    LayoutColors, LayoutNode, LayoutState, Rect,
    context::{BoxModel, FormattingContext, Geometry, LayoutContext},
    mode::{
        LayoutMode,
        block::margin::{MarginCollapseState, calculate_bottom_margin, calculate_top_margin},
        inline::{InlineContext, InlineLayout},
    },
};

pub(crate) mod margin;

#[derive(Debug)]
pub(crate) struct BlockFlowState {
    pub layout_ctx: LayoutContext,
    pub margin_state: MarginCollapseState,
}

impl BlockFlowState {
    pub(crate) fn new(layout_ctx: LayoutContext) -> Self {
        Self {
            layout_ctx,
            margin_state: MarginCollapseState::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ChildLayoutResult {
    node_ids: Vec<LayoutNodeId>,
    _node_dimensions: Vec<Rect>,

    /// The container of the nodes, i.e., the `X` and `Y` are the initial points
    /// and the `width` and `height` are the max, for easier access.
    node_container: Rect,
}

pub struct BlockLayout;

impl BlockLayout {
    /// Lays out the node as a block
    ///
    /// <https://www.w3.org/TR/CSS2/box.html#collapsing-margins>
    pub fn layout<'a, 'input>(
        layout_id: &'a LayoutNodeId,
        parent_style: &'a ComputedStyle,
        flow: &mut BlockFlowState,
        state: &mut LayoutState<'_, 'input>,
    ) -> Option<(LayoutNodeId, Rect)> {
        let box_node = &state.input.box_tree[layout_id];
        let style = &*box_node.style;

        if enabled!(Level::TRACE)
            && let Some(node_id) = box_node.node_id
            && let Some(element) = &state.input.dom[node_id].data.as_element()
        {
            trace!(%element.tag, "↓ Top-down")
        }

        if style.position.is_out_of_flow() && !flow.layout_ctx.is_deferred() {
            let containing_block = if style.position == Position::Fixed {
                Some(flow.layout_ctx.containing_block())
            } else {
                None
            };

            state
                .position_ctx
                .defer(layout_id, containing_block, flow.margin_state);

            return None;
        }

        let box_model = Geometry::resolve_box_model(style, flow.layout_ctx.containing_block().width);
        let establishes_bfc = FormattingContext::establishes_bfc(box_node, parent_style, style, state.input.dom);

        let width = Self::calculate_width(style, &box_model, flow.layout_ctx.containing_block().width);

        let x = Self::calculate_x(style, &flow.layout_ctx, &box_model, width);
        flow.layout_ctx.cursor().x = x;

        let has_top_fence = Geometry::has_top_fence(style, flow.layout_ctx.containing_block().width);
        let has_bottom_fence = Geometry::has_bottom_fence(style, flow.layout_ctx.containing_block().width);

        let has_block_child = box_node
            .children
            .iter()
            .any(|child| matches!(LayoutMode::new(&state.input.box_tree[child]), LayoutMode::Block));

        let collapsed_top = calculate_top_margin(has_block_child, has_top_fence, &mut flow.margin_state, &box_model);
        flow.layout_ctx.cursor().y += collapsed_top;

        let avaliable_child_height =
            Geometry::calculate_height(style, &box_model, f64::INFINITY, flow.layout_ctx.containing_block().height);

        if style.position == Position::Static {
            flow.layout_ctx
                .set_positioned_containing_block(flow.layout_ctx.positioned_containing_block());
        } else {
            let nearest_positioned_ancestor = flow.layout_ctx.positioned_containing_block();
            let rect = Rect::new(nearest_positioned_ancestor.x, nearest_positioned_ancestor.y, width, 0.0);

            state.position_ctx.push_parent(layout_id, rect);
            flow.layout_ctx.set_positioned_containing_block(rect);
        }

        let child_start_y = flow.layout_ctx.containing_block().y + flow.layout_ctx.cursor().y;
        let mut child_flow = BlockFlowState {
            layout_ctx: flow.layout_ctx.child_context(
                Rect {
                    x: x + box_model.padding.left + box_model.border.left,
                    y: child_start_y + box_model.padding.top + box_model.border.top,
                    width,
                    height: avaliable_child_height,
                },
                flow.layout_ctx.is_deferred(),
            ),
            margin_state: flow.margin_state,
        };

        let child_layout_result = Self::layout_children(&box_node.children, style, &mut child_flow, state);

        if enabled!(Level::TRACE)
            && let Some(node_id) = box_node.node_id
            && let Some(element) = &state.input.dom[node_id].data.as_element()
        {
            trace!(%element.tag, "↑ Bottom-up")
        }

        let node_y =
            if style.height.is_auto() && has_block_child && !flow.margin_state.top_collapsed && !establishes_bfc {
                child_layout_result.node_container.y
            } else {
                Self::calculate_y(style, &flow.layout_ctx)
            };

        let collapsed_bottom = calculate_bottom_margin(
            establishes_bfc,
            has_bottom_fence,
            &mut child_flow.margin_state,
            &mut flow.margin_state,
            &box_model,
        );

        let raw_height = if style.height.is_auto() {
            // TODO: MIN-HEIGHT

            if flow.margin_state.bottom_collapsed || establishes_bfc {
                child_layout_result.node_container.height + collapsed_bottom + child_layout_result.node_container.y
                    - (box_model.padding.top + box_model.border.top + node_y)
            } else {
                child_layout_result.node_container.height + collapsed_bottom
            }
        } else {
            Self::calculate_height(
                style,
                &box_model,
                child_layout_result.node_container.height,
                flow.layout_ctx.containing_block().height,
            )
        }
        .max(0.0);

        if matches!(style.float, Float::None) {
            flow.layout_ctx.cursor().y += raw_height;
        }

        let y_adj = state
            .float_ctx
            .clear_y(style.clear, style.writing_mode, node_y);
        flow.layout_ctx.cursor().y += y_adj - node_y;

        let node_dimensions = Rect::new(flow.layout_ctx.cursor().x, y_adj, width, raw_height);

        state.float_ctx.add_float(node_dimensions, style);

        let colors = LayoutColors::from(style);
        let node = LayoutNode::builder(*layout_id)
            .block_formatting_context(establishes_bfc)
            .border(box_model.border)
            .children(child_layout_result.node_ids)
            .colors(colors)
            .cursor(style.cursor)
            .dimensions(node_dimensions)
            .margin(box_model.margin)
            .maybe_node_id(box_node.node_id)
            .padding(box_model.padding)
            .position(style.position)
            .build();

        state.nodes[layout_id.index()] = Some(node);

        let final_dimension_with_padding = Rect::new(
            flow.layout_ctx.cursor().x,
            y_adj,
            width + box_model.padding.horizontal() + box_model.border.horizontal(),
            raw_height + box_model.padding.vertical() + box_model.border.vertical(),
        );

        flow.layout_ctx.cursor().y += box_model.padding.vertical() + box_model.border.vertical();

        Some((*layout_id, final_dimension_with_padding))
    }

    fn layout_children<'input>(
        children: &'input [LayoutNodeId],
        parent_style: &'input ComputedStyle,
        child_flow: &mut BlockFlowState,
        state: &mut LayoutState<'_, 'input>,
    ) -> ChildLayoutResult {
        let mut node_dimensions = Vec::with_capacity(children.len());
        let mut node_ids = Vec::with_capacity(children.len());
        let mut node_container: Option<Rect> = None;

        if children.is_empty() {
            return ChildLayoutResult {
                node_ids,
                _node_dimensions: node_dimensions,
                node_container: Rect::default(),
            };
        }

        match LayoutMode::new(&state.input.box_tree[&children[0]]) {
            LayoutMode::Inline => {
                let inline_items = InlineLayout::collect_inline_items_from_nodes(
                    child_flow.layout_ctx.containing_block(),
                    state.input,
                    parent_style,
                    children,
                );

                let inline_ctx = InlineContext::new(child_flow.layout_ctx.containing_block());

                let (ids, nodes_size, container) = InlineLayout::layout(state, &inline_items, inline_ctx);

                node_ids.extend(ids);
                node_dimensions.extend(nodes_size);
                if let Some(nc) = &mut node_container {
                    nc.width = nc.width.max(container.width);
                    nc.height = nc.height.max(container.y - nc.y + container.height);
                } else {
                    node_container = Some(container);
                }
            }
            _ => {
                // TODO: Handle Flex and Grid.
                for child_id in children {
                    if let Some((node_id, node_size)) = BlockLayout::layout(child_id, parent_style, child_flow, state) {
                        node_ids.push(node_id);
                        node_dimensions.push(node_size);

                        if let Some(nc) = &mut node_container {
                            nc.width = nc.width.max(node_size.width);
                            nc.height = nc.height.max(node_size.y - nc.y + node_size.height);
                        } else {
                            node_container = Some(node_size);
                        }
                    }
                }
            }
        }

        let final_container = node_container.unwrap_or_default();

        ChildLayoutResult {
            node_ids,
            _node_dimensions: node_dimensions,
            node_container: final_container,
        }
    }

    fn calculate_width(style: &ComputedStyle, box_model: &BoxModel, container_width: f64) -> f64 {
        if style.position.is_out_of_flow() {
            let has_left = !style.left.is_auto() && style.left.to_px(container_width) > 0.0;
            let has_right = !style.right.is_auto() && style.right.to_px(container_width) > 0.0;
            let width_is_auto = style.width == ComputedSize::Auto;

            if has_left && has_right && width_is_auto {
                return container_width - style.left.to_px(container_width) - style.right.to_px(container_width);
            }
        }

        let mut specified_width = Geometry::calculate_width(style, container_width);

        if style.width.is_auto() {
            specified_width -= box_model.padding.horizontal() + box_model.border.horizontal();
        }

        specified_width.max(0.0)
    }

    fn calculate_x(style: &ComputedStyle, ctx: &LayoutContext, box_model: &BoxModel, content_width: f64) -> f64 {
        let container_width = ctx.containing_block().width;
        let has_left = !style.left.is_auto();
        let has_right = !style.right.is_auto();
        let margin_left_px = box_model.margin.left.to_px();
        let margin_right_px = box_model.margin.right.to_px();
        let left_px = style.left.to_px(container_width);
        let right_px = style.right.to_px(container_width);

        let total_width = content_width + box_model.padding.horizontal() + box_model.border.horizontal();
        let normal_x = if style.float == Float::Left {
            ctx.containing_block().x + margin_left_px
        } else if style.float == Float::Right {
            ctx.containing_block().x + container_width - margin_right_px - total_width
        } else if style.margin_left.is_auto() && style.margin_right.is_auto() {
            ctx.containing_block().x + (container_width - total_width) / 2.0
        } else if style.margin_left.is_auto() {
            ctx.containing_block().x + container_width - margin_right_px - total_width
        } else {
            ctx.containing_block().x + margin_left_px
        };

        if style.position.is_out_of_flow() {
            if has_left {
                return ctx.containing_block().x + left_px;
            } else if has_right {
                return ctx.containing_block().x + container_width - right_px - total_width;
            }
        } else if style.position == Position::Relative {
            if has_left {
                return normal_x + left_px;
            } else if has_right {
                return normal_x - right_px;
            }
        }

        normal_x
    }

    fn calculate_y(style: &ComputedStyle, ctx: &LayoutContext) -> f64 {
        let normal_y = ctx.containing_block().y + ctx.cursor_ref().y;

        match style.position {
            Position::Relative | Position::Absolute => {
                let top = style.top.to_px(ctx.containing_block().width);
                let bottom = style.bottom.to_px(ctx.containing_block().width);

                if top > 0.0 {
                    normal_y + top
                } else {
                    normal_y - bottom
                }
            }
            _ => normal_y,
        }
    }

    pub(crate) fn calculate_height(
        style: &ComputedStyle,
        box_model: &BoxModel,
        child_height: f64,
        containing_block_height: f64,
    ) -> f64 {
        if style.position.is_out_of_flow() && !(style.top.is_auto() || style.bottom.is_auto()) {
            let top_px = style.top.to_px(containing_block_height);
            let bottom_px = style.bottom.to_px(containing_block_height);

            if style.height.is_defined() {
                let height =
                    Geometry::calculate_height(style, box_model, child_height, containing_block_height).max(0.0);

                (height - top_px - bottom_px).max(0.0)
            } else if style.height == ComputedSize::Auto {
                (child_height - top_px - bottom_px).max(0.0)
            } else {
                (containing_block_height - top_px - bottom_px).max(0.0)
            }
        } else {
            Geometry::calculate_height(style, box_model, child_height, containing_block_height).max(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::{
        LayoutInput, Margin, TextContext,
        context::{FloatContext, ImageContext, PositionContext},
        primitives::SideOffset,
    };
    use css_display::{BoxNode, BoxTree};
    use css_style::{ComputedMargin, ComputedSize, ComputedStyle};
    use html_dom::{DocumentRoot, DomNode, Element, HtmlTag, NodeData, NodeId, Tag};

    use super::*;

    fn viewport() -> Rect {
        Rect::new(0.0, 0.0, 800.0, 600.0)
    }

    #[test]
    fn test_calculate_x_static() {
        let style = ComputedStyle {
            margin_left: ComputedMargin::Auto,
            margin_right: ComputedMargin::Auto,
            ..Default::default()
        };

        let ctx = LayoutContext::new(viewport());

        let box_model = BoxModel {
            border: SideOffset::zero(),
            margin: Margin::zero(),
            padding: SideOffset::zero(),
        };

        let content_width = 400.0;

        let x = BlockLayout::calculate_x(&style, &ctx, &box_model, content_width);

        assert_eq!(x, 200.0);
    }

    #[test]
    fn test_calculate_x_float_left() {
        let style = ComputedStyle {
            float: Float::Left,
            ..Default::default()
        };

        let ctx = LayoutContext::new(viewport());

        let box_model = BoxModel {
            border: SideOffset::zero(),
            margin: Margin::zero(),
            padding: SideOffset::zero(),
        };
        let content_width = 200.0;

        let x = BlockLayout::calculate_x(&style, &ctx, &box_model, content_width);

        assert_eq!(x, 0.0);
    }

    #[test]
    fn test_calculate_x_float_right() {
        let style = ComputedStyle {
            float: Float::Right,
            ..Default::default()
        };

        let ctx = LayoutContext::new(viewport());

        let box_model = BoxModel {
            border: SideOffset::zero(),
            margin: Margin::zero(),
            padding: SideOffset::zero(),
        };
        let content_width = 200.0;

        let x = BlockLayout::calculate_x(&style, &ctx, &box_model, content_width);

        assert_eq!(x, 600.0);
    }

    #[test]
    fn test_calculate_x_absolute_left_precedence_over_right() {
        let style = ComputedStyle {
            position: Position::Absolute,
            left: 50.0.into(),
            right: 30.0.into(),
            ..Default::default()
        };

        let ctx = LayoutContext::new(viewport());

        let box_model = BoxModel {
            border: SideOffset::zero(),
            margin: Margin::zero(),
            padding: SideOffset::zero(),
        };

        let x = BlockLayout::calculate_x(&style, &ctx, &box_model, 200.0);

        assert_eq!(x, 50.0);
    }

    #[test]
    fn test_calculate_x_fixed_right_when_left_auto() {
        let style = ComputedStyle {
            position: Position::Fixed,
            right: 30.0.into(),
            left: ComputedMargin::Auto,
            ..Default::default()
        };

        let ctx = LayoutContext::new(viewport());

        let box_model = BoxModel {
            border: SideOffset::zero(),
            margin: Margin::zero(),
            padding: SideOffset::zero(),
        };

        let x = BlockLayout::calculate_x(&style, &ctx, &box_model, 200.0);

        assert_eq!(x, 570.0);
    }

    #[test]
    fn test_calculate_x_relative_left_offsets_from_normal_position() {
        let style = ComputedStyle {
            position: Position::Relative,
            left: 25.0.into(),
            ..Default::default()
        };

        let ctx = LayoutContext::new(viewport());

        let margin = Margin {
            left: 40.0.into(),
            ..Margin::zero()
        };
        let box_model = BoxModel {
            border: SideOffset::zero(),
            margin,
            padding: SideOffset::zero(),
        };

        let x = BlockLayout::calculate_x(&style, &ctx, &box_model, 200.0);

        assert_eq!(x, 65.0);
    }

    #[test]
    fn test_calculate_x_relative_right_offsets_from_normal_position() {
        let style = ComputedStyle {
            position: Position::Relative,
            right: 30.0.into(),
            ..Default::default()
        };

        let ctx = LayoutContext::new(viewport());

        let margin = Margin {
            left: 40.0.into(),
            ..Margin::zero()
        };
        let box_model = BoxModel {
            border: SideOffset::zero(),
            margin,
            padding: SideOffset::zero(),
        };

        let x = BlockLayout::calculate_x(&style, &ctx, &box_model, 200.0);

        assert_eq!(x, 10.0);
    }

    #[test]
    fn test_calculate_height_absolute_auto_with_top_and_bottom() {
        let style = ComputedStyle {
            position: Position::Absolute,
            top: 20.0.into(),
            bottom: 30.0.into(),
            height: ComputedSize::Auto,
            ..Default::default()
        };

        let box_model = BoxModel::default();
        let height = BlockLayout::calculate_height(&style, &box_model, 0.0, 600.0);
        assert_eq!(height, 0.0);
    }

    #[test]
    fn test_calculate_height_fixed_100_percent_with_top_and_bottom() {
        let style = ComputedStyle {
            position: Position::Fixed,
            top: 10.0.into(),
            bottom: 40.0.into(),
            height: ComputedSize::Percentage(1.0),
            ..Default::default()
        };

        let box_model = BoxModel::default();
        let height = BlockLayout::calculate_height(&style, &box_model, 0.0, 600.0);
        assert_eq!(height, 550.0);
    }

    #[test]
    fn test_calculate_height_relative_auto_ignores_top_and_bottom() {
        let style = ComputedStyle {
            position: Position::Relative,
            top: 30.0.into(),
            bottom: 20.0.into(),
            height: ComputedSize::Auto,
            ..Default::default()
        };

        let box_model = BoxModel::default();
        let height = BlockLayout::calculate_height(&style, &box_model, 0.0, 600.0);
        assert_eq!(height, 0.0);
    }

    #[test]
    fn test_layout_empty() {
        let img_ctx = ImageContext::new();
        let mut float_ctx = FloatContext::new();
        let mut flow = BlockFlowState::new(LayoutContext::new(viewport()));
        let mut position_ctx = PositionContext::new(viewport());

        let mut text_ctx = TextContext::default();
        let dom_tree = DocumentRoot {
            nodes: vec![DomNode {
                id: NodeId(0),
                data: NodeData::Element(Element::new(Tag::Html(HtmlTag::Html), HashSet::new(), HashMap::new())),
                children: vec![],
                parent: None,
            }],
            root_nodes: vec![NodeId(0)],
        };
        let style = ComputedStyle::default();
        let box_node = BoxNode::new(None, LayoutNodeId::new(0), &NodeId(0), &style, vec![]);

        let box_tree = BoxTree {
            nodes: vec![box_node],
            root_nodes: vec![LayoutNodeId::new(0)],
            dom_to_layout: vec![Some(LayoutNodeId::new(0))],
        };

        let mut input = LayoutInput {
            dom: &dom_tree,
            box_tree: &box_tree,
            text: &mut text_ctx,
            image: &img_ctx,
        };
        let mut nodes = vec![None; input.box_tree.nodes.len()];
        let layout_node = {
            let mut state = LayoutState::new(&mut nodes, &mut input, &mut position_ctx, &mut float_ctx);
            BlockLayout::layout(&LayoutNodeId::new(0), &ComputedStyle::default(), &mut flow, &mut state)
        };

        let (layout_node_id, _) = layout_node.unwrap();
        let layout_node = &nodes[layout_node_id.index()].clone().unwrap();

        assert_eq!(nodes.len(), 1);
        assert_eq!(layout_node.layout_id, LayoutNodeId::new(0));
        assert_eq!(layout_node.dimensions.x, 0.0);
        assert_eq!(layout_node.dimensions.y, 0.0);
        assert_eq!(layout_node.dimensions.width, 800.0);
        assert_eq!(layout_node.dimensions.height, 0.0);
        assert_eq!(layout_node.children.len(), 0);
    }
}
