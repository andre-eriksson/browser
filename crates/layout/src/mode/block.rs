use css_display::LayoutNodeId;
use css_style::{ComputedSize, ComputedStyle, Position};
use css_values::display::Float;

use crate::{
    LayoutColors, LayoutNode, Rect,
    context::{BoxModel, FormattingContext, Geometry, LayoutContext, PositionContext},
    engine::LayoutInput,
    mode::{
        LayoutMode,
        inline::{InlineContext, InlineLayout},
    },
    primitives::Size,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct MarginCollapsing {
    pub max_positive: f64,
    pub max_negative: f64,
}

impl MarginCollapsing {
    fn add(&mut self, margin: f64) {
        if margin >= 0.0 {
            self.max_positive = self.max_positive.max(margin);
        } else {
            self.max_negative = self.max_negative.min(margin);
        }
    }

    fn add_collapsed(&mut self, other: &MarginCollapsing) {
        self.add(other.max_positive);
        self.add(other.max_negative);
    }

    fn flush(&mut self) -> f64 {
        let collapsed = self.max_positive + self.max_negative;
        self.max_positive = 0.0;
        self.max_negative = 0.0;
        collapsed
    }
}

#[derive(Debug, Clone, Default)]
pub struct BlockContext {
    pub defer_top_margin: bool,
    pub collapsed_margin: MarginCollapsing,
    pub deferred_top_margin: Option<MarginCollapsing>,
}

impl BlockContext {
    pub fn new() -> Self {
        Self {
            defer_top_margin: false,
            collapsed_margin: MarginCollapsing {
                max_positive: 0.0,
                max_negative: 0.0,
            },
            deferred_top_margin: None,
        }
    }
}

struct BlockChildContext {
    pub layout_ctx: LayoutContext,
    pub block_ctx: BlockContext,
    pub collapse_first_child_top: bool,
}
pub struct BlockLayout;

impl BlockLayout {
    pub fn layout(
        nodes: &mut Vec<Option<LayoutNode>>,
        layout_id: &LayoutNodeId,
        parent_style: &ComputedStyle,
        input: &mut LayoutInput<'_>,
        ctx: &mut LayoutContext,
        position_ctx: &mut PositionContext,
        current_block: &mut BlockContext,
    ) -> Option<(LayoutNodeId, Size)> {
        let box_node = &input.box_tree[layout_id];
        let style = &*box_node.style;

        if style.position.is_out_of_flow() && !ctx.is_deferred() {
            let containing_block = if style.position == Position::Fixed {
                Some(ctx.containing_block())
            } else {
                None
            };

            position_ctx.defer(layout_id, containing_block, current_block.clone());

            return None;
        }

        let box_model = Geometry::resolve_box_model(style, ctx.containing_block().width);
        let establishes_bfc = FormattingContext::establishes_bfc(box_node, parent_style, style, input.dom);

        let width = Self::calculate_width(style, ctx.containing_block().width, &box_model);
        let x = Self::calculate_x(style, ctx, &box_model, width);
        ctx.cursor().x = x;

        let has_top_fence = Geometry::has_top_fence(style, ctx.containing_block().width);
        let has_block_child = box_node
            .children
            .iter()
            .any(|child| matches!(LayoutMode::new(&input.box_tree[child]), LayoutMode::Block));
        let can_collapse_top_with_child = has_block_child && !has_top_fence && !establishes_bfc;

        current_block.deferred_top_margin = None;

        Self::apply_top_margin(has_top_fence, can_collapse_top_with_child, current_block, &box_model, ctx);

        let avaliable_child_height =
            Geometry::calculate_height(style, &box_model, ctx.containing_block().height, ctx.containing_block().height);

        if style.position == Position::Static {
            ctx.set_positioned_containing_block(ctx.positioned_containing_block());
        } else {
            let nearest_positioned_ancestor = ctx.positioned_containing_block();
            let rect = Rect::new(nearest_positioned_ancestor.x, nearest_positioned_ancestor.y, width, 0.0);

            position_ctx.push_parent(layout_id, rect);
            ctx.set_positioned_containing_block(rect);
        }

        let child_start_y = ctx.containing_block().y + ctx.cursor().y;

        let mut child_ctx = BlockChildContext {
            layout_ctx: ctx.child_context(
                Rect {
                    x,
                    y: child_start_y,
                    width,
                    height: avaliable_child_height,
                },
                ctx.is_deferred(),
            ),
            block_ctx: BlockContext::new(),
            collapse_first_child_top: can_collapse_top_with_child,
        };

        let (ids, deferred_child_top) =
            Self::layout_children(nodes, &box_node.children, style, input, position_ctx, &mut child_ctx);

        let applied_top_margin = Self::resolve_deferred_top(
            nodes,
            can_collapse_top_with_child,
            current_block,
            deferred_child_top,
            &ids,
            ctx,
        );

        let can_collapse_bottom = !Geometry::has_bottom_fence(style, ctx.containing_block().width) && !establishes_bfc;

        Self::apply_bottom_margin(can_collapse_bottom, &mut child_ctx.block_ctx, current_block, &box_model);

        let content_height = Self::calculate_height(
            style,
            &box_model,
            (child_ctx.layout_ctx.cursor().y - applied_top_margin).max(0.0),
            ctx.containing_block().height,
        );

        let node_y = Self::calculate_start_y(style, ctx);

        let colors = LayoutColors::from(style);
        let node = LayoutNode::builder(*layout_id)
            .block_formatting_context(establishes_bfc)
            .border(box_model.border)
            .children(ids)
            .colors(colors)
            .cursor(style.cursor)
            .dimensions(Rect::new(ctx.cursor().x, node_y, width, content_height))
            .margin(box_model.margin)
            .maybe_node_id(box_node.node_id)
            .padding(box_model.padding)
            .position(style.position)
            .build();

        nodes[layout_id.index()] = Some(node);

        ctx.cursor().y += content_height + box_model.padding.vertical() + box_model.border.vertical();

        Some((*layout_id, Size::new(width, content_height)))
    }

    fn layout_children<'a>(
        nodes: &mut Vec<Option<LayoutNode>>,
        children: &'a [LayoutNodeId],
        parent_style: &'a ComputedStyle,
        input: &mut LayoutInput<'a>,
        position_ctx: &mut PositionContext,
        child_ctx: &mut BlockChildContext,
    ) -> (Vec<LayoutNodeId>, Option<MarginCollapsing>) {
        let mut node_ids = Vec::with_capacity(children.len());
        let mut deferred_child_top = None;

        if children.is_empty() {
            return (node_ids, deferred_child_top);
        }

        match LayoutMode::new(&input.box_tree[&children[0]]) {
            LayoutMode::Inline => {
                let inline_items = InlineLayout::collect_inline_items_from_nodes(
                    child_ctx.layout_ctx.containing_block(),
                    input,
                    parent_style,
                    children,
                );

                let inline_ctx = InlineContext::new(child_ctx.layout_ctx.containing_block());

                let (ids, nodes_size) = InlineLayout::layout(
                    nodes,
                    input,
                    &inline_items,
                    &mut child_ctx.layout_ctx,
                    position_ctx,
                    inline_ctx,
                );

                child_ctx.layout_ctx.cursor().y += nodes_size.height;
                node_ids.extend(ids);
            }
            _ => {
                // TODO: Handle Flex and Grid.
                for child_id in children {
                    let is_first = node_ids.is_empty();
                    let defer = child_ctx.collapse_first_child_top && is_first;
                    child_ctx.block_ctx.defer_top_margin = defer;

                    if is_first {
                        let box_node = &input.box_tree[child_id];
                        let style = &*box_node.style;

                        if Geometry::has_top_fence(style, child_ctx.layout_ctx.containing_block().width) {
                            let box_model =
                                Geometry::resolve_box_model(style, child_ctx.layout_ctx.containing_block().width);
                            let flushed = child_ctx.block_ctx.collapsed_margin.flush();
                            child_ctx.layout_ctx.cursor().y += flushed + box_model.padding.top + box_model.border.top;
                        }
                    }

                    if let Some((node_id, _)) = BlockLayout::layout(
                        nodes,
                        child_id,
                        parent_style,
                        input,
                        &mut child_ctx.layout_ctx,
                        position_ctx,
                        &mut child_ctx.block_ctx,
                    ) {
                        if defer {
                            deferred_child_top = child_ctx.block_ctx.deferred_top_margin.take();
                        }
                        node_ids.push(node_id);
                    }
                }
            }
        }

        (node_ids, deferred_child_top)
    }

    fn offset_node_y(nodes: &mut Vec<Option<LayoutNode>>, id: &LayoutNodeId, delta_y: f64) {
        let Some(mut node) = std::mem::take(&mut nodes[id.index()]) else {
            return;
        };

        node.dimensions.y += delta_y;
        for child_id in &node.children {
            Self::offset_node_y(nodes, child_id, delta_y);
        }

        nodes[id.index()] = Some(node);
    }

    fn calculate_width(style: &ComputedStyle, container_width: f64, box_model: &BoxModel) -> f64 {
        if style.position.is_out_of_flow() {
            let has_left = !style.left.is_auto() && style.left.to_px(container_width) > 0.0;
            let has_right = !style.right.is_auto() && style.right.to_px(container_width) > 0.0;
            let width_is_auto = style.width == ComputedSize::Auto;

            if has_left && has_right && width_is_auto {
                return container_width - style.left.to_px(container_width) - style.right.to_px(container_width);
            }
        }

        let specified_width = Geometry::calculate_width(style, container_width);
        if style.width == ComputedSize::Auto {
            (specified_width - box_model.padding.horizontal() - box_model.border.horizontal()).max(0.0)
        } else {
            specified_width
        }
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

    fn calculate_start_y(style: &ComputedStyle, ctx: &LayoutContext) -> f64 {
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

    fn apply_top_margin(
        has_top_fence: bool,
        can_collapse_top_with_child: bool,
        current_block: &mut BlockContext,
        box_model: &BoxModel,
        ctx: &mut LayoutContext,
    ) {
        if has_top_fence {
            let flushed = current_block.collapsed_margin.flush();
            ctx.cursor().y += flushed;
        } else {
            current_block
                .collapsed_margin
                .add(box_model.margin.top.to_px());

            if !can_collapse_top_with_child {
                if current_block.defer_top_margin {
                    current_block.deferred_top_margin = Some(current_block.collapsed_margin);
                    current_block.collapsed_margin.flush();
                } else {
                    let collapsed = current_block.collapsed_margin.flush();
                    ctx.cursor().y += collapsed;
                }
            }
        }
    }

    fn apply_bottom_margin(
        can_collapse_bottom: bool,
        child_block: &mut BlockContext,
        current_block: &mut BlockContext,
        box_model: &BoxModel,
    ) {
        // The bottom margin of an in-flow block box with a 'height' of 'auto' and a 'min-height' of zero collapses
        //   with its last in-flow block-level child's bottom margin if the box has no bottom padding and no bottom
        //   border and the child's bottom margin does not collapse with a top margin that has clearance.
        //
        // A box's own margins collapse if the 'min-height' property is zero, and it has neither top or bottom borders
        //   nor top or bottom padding, and it has a 'height' of either 0 or 'auto', and it does not contain a line box,
        //   and all of its in-flow children's margins (if any) collapse.
        if can_collapse_bottom {
            child_block
                .collapsed_margin
                .add(box_model.margin.bottom.to_px());
            current_block
                .collapsed_margin
                .add_collapsed(&child_block.collapsed_margin);
        } else {
            current_block
                .collapsed_margin
                .add(box_model.margin.bottom.to_px());
        }
    }

    fn resolve_deferred_top(
        nodes: &mut Vec<Option<LayoutNode>>,
        can_collapse_top_with_child: bool,
        current_block: &mut BlockContext,
        deferred_child_top: Option<MarginCollapsing>,
        ids: &[LayoutNodeId],
        ctx: &mut LayoutContext,
    ) -> f64 {
        if !can_collapse_top_with_child {
            return 0.0;
        }

        if let Some(child_top) = deferred_child_top {
            current_block.collapsed_margin.add_collapsed(&child_top);
        }

        if current_block.defer_top_margin {
            current_block.deferred_top_margin = Some(current_block.collapsed_margin);
            current_block.collapsed_margin.flush();
            return 0.0;
        }

        let collapsed = current_block.collapsed_margin.flush();

        if collapsed != 0.0 {
            for child_id in ids {
                Self::offset_node_y(nodes, child_id, collapsed);
            }
        }
        ctx.cursor().y += collapsed;
        collapsed
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::{Margin, TextContext, context::ImageContext, primitives::SideOffset};
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
        let mut ctx = LayoutContext::new(viewport());
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
        let mut block_ctx = BlockContext {
            defer_top_margin: false,
            collapsed_margin: MarginCollapsing {
                max_positive: 0.0,
                max_negative: 0.0,
            },
            deferred_top_margin: None,
        };

        let mut nodes = vec![None; input.box_tree.nodes.len()];

        let layout_node = BlockLayout::layout(
            &mut nodes,
            &LayoutNodeId::new(0),
            &ComputedStyle::default(),
            &mut input,
            &mut ctx,
            &mut position_ctx,
            &mut block_ctx,
        );

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
