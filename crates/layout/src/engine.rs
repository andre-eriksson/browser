use css_style::{ComputedStyle, Position, StyleTree, StyledNode};
use css_values::display::{BoxDisplay, InsideDisplay};
use html_dom::{DocumentRoot, NodeId};

use crate::{
    context::ImageContext,
    layout::{LayoutContext, LayoutNode, LayoutTree},
    mode::{
        block::BlockLayout,
        inline::{InlineContext, InlineLayout},
    },
    position::PositionContext,
    primitives::Rect,
    text::TextContext,
};

const EPSILON: f32 = 0.1;

/// Layout mode determines how children are positioned
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LayoutMode {
    Block,
    Inline,
    Flex, // TODO: implement
    Grid, // TODO: implement
}

impl LayoutMode {
    pub fn new(styled_node: &StyledNode) -> Option<Self> {
        if styled_node.style.display.box_display() == Some(BoxDisplay::None) {
            return None;
        }

        if styled_node.style.position.is_out_of_flow() {
            return Some(Self::Block);
        }

        match styled_node.style.display.inside() {
            Some(InsideDisplay::Flex) => Some(Self::Flex),
            Some(InsideDisplay::Grid) => Some(Self::Grid),
            _ => Some(Self::Block),
        }
    }
}

pub struct LayoutEngine;

impl LayoutEngine {
    /// Compute layout for an entire style tree, using known image dimensions
    /// from `image_ctx` so that previously-decoded images are laid out at their
    /// intrinsic size rather than a placeholder.
    ///
    /// This is the core of the **relayout** system: after an image is fetched
    /// and decoded the caller stores its `(width, height)` in an
    /// [`ImageContext`], then calls this method to produce a fresh
    /// [`LayoutTree`] where the image and all of its siblings / ancestors have
    /// been repositioned correctly.
    pub fn compute_layout(
        dom_tree: &DocumentRoot,
        style_tree: &StyleTree,
        viewport: Rect,
        text_ctx: &mut TextContext,
        image_ctx: &ImageContext,
    ) -> LayoutTree {
        let mut position_ctx = PositionContext::new(viewport);
        let mut ctx = LayoutContext::new(viewport, image_ctx, &mut position_ctx);

        let mut total_height = 0.0;
        let mut max_width: f32 = 0.0;
        let mut root_nodes = Vec::new();

        for styled_node in &style_tree.root_nodes {
            ctx.block_cursor.y = total_height;

            let pos_count_before = ctx.position_ctx().position_count();

            let mut node = match Self::layout_node(dom_tree, styled_node, &mut ctx, text_ctx) {
                Some(node) => node,
                None => continue, // For `display: none`
            };

            let top_margin = node.margin.top;
            let bottom_margin = node.margin.bottom;

            Self::offset_children_y(&mut node.children, top_margin);
            ctx.position_ctx()
                .offset_positions_since(pos_count_before, top_margin);

            node.dimensions.height += top_margin + bottom_margin;

            total_height += node.dimensions.height;
            max_width = max_width.max(node.dimensions.width);

            root_nodes.push(node);
        }

        for mut defered_node in ctx
            .position_ctx()
            .resolve_all(dom_tree, image_ctx, text_ctx)
        {
            Self::offset_children_y(&mut defered_node.children, defered_node.margin.top);

            root_nodes.push(defered_node);
        }

        LayoutTree {
            root_nodes,
            content_height: total_height,
            content_width: max_width,
        }
    }

    /// Recursively offset all children's y positions
    fn offset_children_y(children: &mut [LayoutNode], offset: f32) {
        for child in children.iter_mut() {
            if child.position.is_out_of_flow() {
                continue;
            }

            child.dimensions.y += offset;
            Self::offset_children_y(&mut child.children, offset);
        }
    }

    /// Compute layout for a single node and its descendants
    pub(crate) fn layout_node(
        dom_tree: &DocumentRoot,
        styled_node: &StyledNode,
        ctx: &mut LayoutContext,
        text_ctx: &mut TextContext,
    ) -> Option<LayoutNode> {
        let layout_mode = LayoutMode::new(styled_node)?;

        match layout_mode {
            LayoutMode::Block => BlockLayout::layout(dom_tree, styled_node, ctx, text_ctx),
            LayoutMode::Flex => BlockLayout::layout(dom_tree, styled_node, ctx, text_ctx), // TODO: implement flex layout
            LayoutMode::Grid => BlockLayout::layout(dom_tree, styled_node, ctx, text_ctx), // TODO: implement grid layout
            _ => unreachable!("Only block layout should be supported for single nodes right now"),
        }
    }

    pub(crate) fn layout_nodes(
        dom_tree: &DocumentRoot,
        styled_nodes: &[&StyledNode],
        mode: LayoutMode,
        parent_style: &ComputedStyle,
        containing_block: Rect,
        ctx: &mut LayoutContext,
        text_ctx: &mut TextContext,
    ) -> (Vec<LayoutNode>, Rect) {
        match mode {
            LayoutMode::Inline => {
                let inline_items = InlineLayout::collect_inline_items_from_nodes(
                    dom_tree,
                    parent_style,
                    styled_nodes,
                    ctx.image_ctx(),
                );
                let inline_ctx = InlineContext::new(containing_block);

                InlineLayout::layout(dom_tree, &inline_items, ctx, text_ctx, inline_ctx)
            }
            _ => todo!("Only inline layout is supported for collections of nodes right now"),
        }
    }

    /// Relayout a single node and its ancestors, updating the layout tree in place.
    pub fn relayout_node(
        node_id: NodeId,
        viewport: Rect,
        layout_tree: &mut LayoutTree,
        style_tree: &StyleTree,
        dom_tree: &DocumentRoot,
        text_ctx: &mut TextContext,
        image_ctx: &ImageContext,
    ) {
        let ancestors: Vec<NodeId> = dom_tree
            .ancestors(&node_id)
            .into_iter()
            .map(|n| n.id)
            .collect();

        let Some(&dirty_parent_id) = ancestors.first() else {
            return;
        };
        let Some(parent_path) = layout_tree.find_path(dirty_parent_id) else {
            return;
        };

        let old_layout = layout_tree.node_at(&parent_path).unwrap();
        let old_height = old_layout.dimensions.height;

        let Some(styled_node) = style_tree.find_node(&dirty_parent_id) else {
            return;
        };

        let mut position_ctx = PositionContext::new(viewport);
        let mut ctx = LayoutContext::new(old_layout.dimensions, image_ctx, &mut position_ctx);
        ctx.position_ctx().update_viewport(viewport);

        let mut new_node = match Self::layout_node(dom_tree, styled_node, &mut ctx, text_ctx) {
            Some(node) => node,
            None => return, // For `display: none`
        };

        Self::offset_children_y(&mut new_node.children, new_node.margin.top);

        let new_height = new_node.dimensions.height;
        let delta = new_height - old_height;

        *layout_tree.node_at_mut(&parent_path).unwrap() = new_node;

        if delta.abs() < EPSILON {
            return;
        }

        for ancestor_id in ancestors.iter().skip(1) {
            let Some(ancestor_path) = layout_tree.find_path(*ancestor_id) else {
                break;
            };
            let ancestor = layout_tree.node_at_mut(&ancestor_path).unwrap();

            let prev_id = ancestors[ancestors.iter().position(|id| id == ancestor_id).unwrap() - 1];

            let changed_child_idx = ancestor
                .children
                .iter()
                .position(|child| child.node_id == prev_id);

            if let Some(idx) = changed_child_idx {
                for sibling in ancestor.children[idx + 1..].iter_mut() {
                    Self::shift_y_recursively(sibling, delta);
                }
            }

            if ancestor.is_height_auto {
                ancestor.dimensions.height += delta;
            } else {
                break;
            }
        }

        layout_tree.content_height += delta;
    }

    fn shift_y_recursively(node: &mut LayoutNode, delta: f32) {
        node.dimensions.y += delta;
        for child in node.children.iter_mut() {
            Self::shift_y_recursively(child, delta);
        }
    }

    pub(crate) fn collect_children<'node, F>(
        ctx: &mut LayoutContext,
        parent_node: &'node StyledNode,
        start_idx: &mut usize,
        condition: F,
    ) -> Vec<&'node StyledNode>
    where
        F: Fn(&StyledNode) -> bool,
    {
        let mut collected = Vec::new();
        for child in parent_node.children.iter().skip(*start_idx) {
            if child.style.position.is_out_of_flow() && !ctx.is_deferred() {
                let containing_block = if child.style.position == Position::Fixed {
                    ctx.containing_block()
                } else {
                    ctx.positioned_containing_block()
                };

                ctx.position_ctx().defer(child.clone(), containing_block);
                *start_idx += 1;
                continue;
            }

            if condition(child) {
                collected.push(child);
                *start_idx += 1;
            } else {
                break;
            }
        }

        collected
    }
}

#[cfg(test)]
mod tests {
    use css_style::{ComputedStyle, Display};
    use css_values::display::OutsideDisplay;
    use html_dom::NodeId;

    use super::*;

    fn viewport() -> Rect {
        Rect::new(0.0, 0.0, 800.0, 600.0)
    }

    #[test]
    fn test_layout_mode_none() {
        let styled_node = StyledNode {
            style: ComputedStyle {
                display: Display::from(BoxDisplay::None),
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
                display: Display::from(OutsideDisplay::Block),
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
                display: Display::from(InsideDisplay::Flex),
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
                display: Display::from(InsideDisplay::Grid),
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
                display: Display::from(OutsideDisplay::Block),
                ..Default::default()
            },
            ..StyledNode::new(NodeId(0))
        };

        let img_ctx = ImageContext::new();
        let mut position_ctx = PositionContext::new(viewport());
        let mut ctx = LayoutContext::new(viewport(), &img_ctx, &mut position_ctx);

        let mut text_ctx = TextContext::default();
        let dom_tree = DocumentRoot::default();

        let layout_node = BlockLayout::layout(&dom_tree, &styled_node, &mut ctx, &mut text_ctx).unwrap();

        assert_eq!(layout_node.node_id, styled_node.node_id);
        assert_eq!(layout_node.dimensions.x, 0.0);
        assert_eq!(layout_node.dimensions.y, 0.0);
        assert_eq!(layout_node.dimensions.width, 800.0);
        assert_eq!(layout_node.dimensions.height, 0.0);
        assert_eq!(layout_node.children.len(), 0);
    }
}
