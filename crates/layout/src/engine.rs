use css_style::{ComputedStyle, Display, Position, StyleTree};
use css_values::display::{InsideDisplay, OutsideDisplay};
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

const EPSILON: f64 = 0.1;

/// Layout mode determines how children are positioned
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum LayoutMode {
    #[default]
    Block,
    Inline,
    Flex, // TODO: implement
    Grid, // TODO: implement
}

impl LayoutMode {
    pub fn new(style: &ComputedStyle) -> Option<Self> {
        if style.display.is_none() {
            return None;
        }

        if style.position.is_out_of_flow() {
            return Some(Self::Block);
        }

        if let Display::Normal { outside, inside } = style.display {
            if let Some(val) = inside {
                match val {
                    InsideDisplay::Flex => return Some(Self::Flex),
                    InsideDisplay::Grid => return Some(Self::Grid),
                    _ => {}
                }
            }

            if let Some(val) = outside {
                match val {
                    OutsideDisplay::Block => return Some(Self::Block),
                    OutsideDisplay::Inline => return Some(Self::Inline),
                }
            }
        }

        Some(Self::Block)
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

        let mut total_height = 0.0f64;
        let mut max_width = 0.0f64;
        let mut root_nodes = Vec::new();

        for node_id in &dom_tree.root_nodes {
            ctx.block_cursor.y = total_height;

            let pos_count_before = ctx.position_ctx().position_count();

            let Some(mut node) = Self::layout_node(dom_tree, style_tree, node_id, &mut ctx, text_ctx) else {
                continue;
            };

            let top_margin = node.margin.top.to_px();
            let bottom_margin = node.margin.bottom.to_px();

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
            .resolve_all(dom_tree, style_tree, image_ctx, text_ctx)
        {
            Self::offset_children_y(&mut defered_node.children, defered_node.margin.top.to_px());

            root_nodes.push(defered_node);
        }

        LayoutTree {
            root_nodes,
            content_height: total_height,
            content_width: max_width,
        }
    }

    /// Recursively offset all children's y positions
    fn offset_children_y(children: &mut [LayoutNode], offset: f64) {
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
        style_tree: &StyleTree,
        node_id: &NodeId,
        ctx: &mut LayoutContext,
        text_ctx: &mut TextContext,
    ) -> Option<LayoutNode> {
        let style = &style_tree[node_id];
        let _layout_mode = LayoutMode::new(style)?;

        BlockLayout::layout(node_id, dom_tree, style_tree, ctx, text_ctx)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layout_nodes(
        dom_tree: &DocumentRoot,
        style_tree: &StyleTree,
        node_ids: &[NodeId],
        mode: LayoutMode,
        parent_style: &ComputedStyle,
        containing_block: Rect,
        ctx: &mut LayoutContext,
        text_ctx: &mut TextContext,
    ) -> (Vec<LayoutNode>, Rect) {
        match mode {
            LayoutMode::Inline => {
                let inline_items = InlineLayout::collect_inline_items_from_nodes(
                    containing_block,
                    dom_tree,
                    style_tree,
                    parent_style,
                    node_ids,
                    ctx.image_ctx(),
                );
                let inline_ctx = InlineContext::new(containing_block);

                InlineLayout::layout(dom_tree, style_tree, &inline_items, ctx, text_ctx, inline_ctx)
            }
            _ => todo!("Only inline layout is supported for collections of nodes right now"),
        }
    }

    /// Relayout a single node and its ancestors, updating the layout tree in place.
    ///
    /// # Panics
    /// * If the node or any of its ancestors are not found in the layout tree, which should never happen since the layout tree is built from the DOM tree.
    pub fn relayout_node(
        node_id: NodeId,
        viewport: Rect,
        layout_tree: &mut LayoutTree,
        style_tree: &StyleTree,
        dom_tree: &DocumentRoot,
        text_ctx: &mut TextContext,
        image_ctx: &ImageContext,
    ) {
        let node = &dom_tree[node_id];

        let ancestors: Vec<NodeId> = dom_tree.ancestors(node).into_iter().map(|n| n.id).collect();

        let Some(&dirty_parent_id) = ancestors.first() else {
            return;
        };
        let Some(parent_path) = layout_tree.find_path(dirty_parent_id) else {
            return;
        };

        let old_layout = layout_tree.node_at(&parent_path).unwrap();
        let old_height = old_layout.dimensions.height;

        let mut position_ctx = PositionContext::new(viewport);
        let mut ctx = LayoutContext::new(old_layout.dimensions, image_ctx, &mut position_ctx);
        ctx.position_ctx().update_viewport(viewport);

        let Some(mut new_node) = Self::layout_node(dom_tree, style_tree, &dirty_parent_id, &mut ctx, text_ctx) else {
            return;
        };

        Self::offset_children_y(&mut new_node.children, new_node.margin.top.to_px());

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
                for sibling in &mut ancestor.children[idx + 1..] {
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

    fn shift_y_recursively(node: &mut LayoutNode, delta: f64) {
        node.dimensions.y += delta;
        for child in &mut node.children {
            Self::shift_y_recursively(child, delta);
        }
    }

    pub(crate) fn collect_children<F>(
        ctx: &mut LayoutContext,
        dom_tree: &DocumentRoot,
        style_tree: &StyleTree,
        parent_id: &NodeId,
        start_idx: &mut usize,
        condition: F,
    ) -> Vec<NodeId>
    where
        F: Fn(&NodeId) -> bool,
    {
        let mut collected = Vec::new();
        let parent_node = &dom_tree[parent_id];
        let children = &parent_node.children;

        for child in children.iter().skip(*start_idx) {
            let style = &style_tree[child];

            if style.position.is_out_of_flow() && !ctx.is_deferred() {
                let containing_block = if style.position == Position::Fixed {
                    ctx.containing_block()
                } else {
                    ctx.positioned_containing_block()
                };

                ctx.position_ctx().defer(child, containing_block);
                *start_idx += 1;
                continue;
            }

            if condition(child) {
                collected.push(*child);
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
    use std::collections::{HashMap, HashSet};

    use css_style::{ComputedStyle, Display};
    use css_values::display::{BoxDisplay, OutsideDisplay};
    use html_dom::{DomNode, Element, HtmlTag, NodeData, NodeId, Tag};

    use super::*;

    fn viewport() -> Rect {
        Rect::new(0.0, 0.0, 800.0, 600.0)
    }

    #[test]
    fn test_layout_mode_none() {
        let style = ComputedStyle {
            display: Display::Box(BoxDisplay::None),
            ..Default::default()
        };

        assert_eq!(LayoutMode::new(&style), None);
    }

    #[test]
    fn test_layout_mode_block() {
        let style = ComputedStyle {
            display: Display::from(OutsideDisplay::Block),
            ..Default::default()
        };

        assert_eq!(LayoutMode::new(&style), Some(LayoutMode::Block));
    }

    #[test]
    fn test_layout_mode_inline() {
        let style = ComputedStyle {
            display: Display::from(OutsideDisplay::Inline),
            ..Default::default()
        };

        assert_eq!(LayoutMode::new(&style), Some(LayoutMode::Inline));
    }

    #[test]
    fn test_layout_mode_flex() {
        let style = ComputedStyle {
            display: Display::from(InsideDisplay::Flex),
            ..Default::default()
        };

        assert_eq!(LayoutMode::new(&style), Some(LayoutMode::Flex));
    }

    #[test]
    fn test_layout_mode_grid() {
        let style = ComputedStyle {
            display: Display::from(InsideDisplay::Grid),
            ..Default::default()
        };

        assert_eq!(LayoutMode::new(&style), Some(LayoutMode::Grid));
    }

    #[test]
    fn test_layout_empty() {
        let style = ComputedStyle {
            display: Display::from(OutsideDisplay::Block),
            ..Default::default()
        };

        let style_tree = StyleTree::from(vec![style]);

        let img_ctx = ImageContext::new();
        let mut position_ctx = PositionContext::new(viewport());
        let mut ctx = LayoutContext::new(viewport(), &img_ctx, &mut position_ctx);

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

        let layout_node = BlockLayout::layout(&NodeId(0), &dom_tree, &style_tree, &mut ctx, &mut text_ctx).unwrap();

        assert_eq!(layout_node.node_id, NodeId(0));
        assert_eq!(layout_node.dimensions.x, 0.0);
        assert_eq!(layout_node.dimensions.y, 0.0);
        assert_eq!(layout_node.dimensions.width, 800.0);
        assert_eq!(layout_node.dimensions.height, 0.0);
        assert_eq!(layout_node.children.len(), 0);
    }
}
