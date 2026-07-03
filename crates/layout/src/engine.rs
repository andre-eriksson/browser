use crate::{
    LayoutNode, LayoutTree,
    context::{FloatContext, ImageContext, LayoutContext, PositionContext, TextContext},
    mode::{
        LayoutMode,
        block::{BlockContext, BlockLayout},
        inline::{InlineContext, InlineLayout},
    },
    primitives::Rect,
};
use css_display::{BoxTree, LayoutNodeId};
use css_style::ComputedStyle;
use html_dom::{DocumentRoot, NodeId};

use tracing::trace;

/// Immutable references to the trees — passed everywhere, never mutated
pub struct LayoutInput<'a> {
    pub dom: &'a DocumentRoot,
    pub box_tree: &'a BoxTree<'a>,
    pub text: &'a mut TextContext,
    pub image: &'a ImageContext,
}

impl LayoutTree {
    const EPSILON: f64 = 0.1;

    /// Compute layout for an entire style tree, using known image dimensions
    /// from `image_ctx` so that previously-decoded images are laid out at their
    /// intrinsic size rather than a placeholder.
    ///
    /// This is the core of the **relayout** system: after an image is fetched
    /// and decoded the caller stores its `(width, height)` in an
    /// [`ImageContext`], then calls this method to produce a fresh
    /// [`LayoutTree`] where the image and all of its siblings / ancestors have
    /// been repositioned correctly.
    pub fn compute_layout(input: &mut LayoutInput, viewport: Rect) -> Self {
        let mut position_ctx = PositionContext::new(viewport);
        let mut float_ctx = FloatContext::new();

        let mut ctx = LayoutContext::new(viewport);
        let mut total_height = 0.0f64;
        let mut max_width = 0.0f64;
        let mut root_nodes = Vec::with_capacity(input.box_tree.root_nodes.len());
        let mut nodes = vec![None; input.box_tree.nodes.len()];

        for layout_id in &input.box_tree.root_nodes {
            let mut block_ctx = BlockContext::default();

            let Some((id, size)) = BlockLayout::layout(
                &mut nodes,
                layout_id,
                &ComputedStyle::default(),
                input,
                &mut ctx,
                &mut position_ctx,
                &mut block_ctx,
                &mut float_ctx,
            ) else {
                continue;
            };

            total_height += size.height;
            max_width = max_width.max(size.width);

            root_nodes.push(id);
        }

        let mut tree = Self {
            root_nodes,
            nodes,
            content_height: total_height,
            content_width: max_width,
        };

        trace!("Initial layout complete, resolving deferred positions...");
        position_ctx.resolve_all(input, &mut tree);

        debug_assert!(
            tree.nodes
                .iter()
                .flatten()
                .is_sorted_by(|a, b| a.layout_id.index() < b.layout_id.index())
        );

        tree
    }

    /// Relayout a single node and its ancestors, updating the layout tree in place.
    ///
    /// # Panics
    /// * If the node or any of its ancestors are not found in the layout tree, which should never happen since the layout tree is built from the DOM tree.
    pub fn relayout_node<'css>(
        node_id: &NodeId,
        viewport: Rect,
        layout_tree: &mut LayoutTree,
        input: &mut LayoutInput<'css>,
    ) {
        let Some(layout_id) = input.box_tree.dom_to_layout[node_id.index()] else {
            panic!("Layout ID not found for node_id: {:?}", node_id);
        };

        let Some(old_node) = &layout_tree.nodes[layout_id.index()] else {
            panic!("Layout node not found for layout_id: {:?}", layout_id);
        };

        let box_node = &input.box_tree[&layout_id];

        let ancestors: Vec<LayoutNodeId> = input
            .box_tree
            .ancestors(box_node)
            .into_iter()
            .map(|n| n.layout_id)
            .collect();

        let mut path = Vec::with_capacity(ancestors.len() + 1);
        path.push(layout_id);
        path.extend(ancestors);

        let old_height = old_node.dimensions.height;

        // TODO: Restore old contexts
        let mut position_ctx = PositionContext::new(viewport);
        let mut float_ctx = FloatContext::new();

        let mut ctx = LayoutContext::new(old_node.dimensions);

        let style = &*box_node.style;
        let mode = LayoutMode::new(box_node);

        let containing_block = if let Some(parent) = box_node.parent_id
            && let Some(parent_node) = &layout_tree.nodes[parent.index()]
        {
            parent_node.dimensions
        } else {
            old_node.dimensions
        };

        let (nodes, _, node_container) = match mode {
            LayoutMode::Inline => {
                let inline_items =
                    InlineLayout::collect_inline_items_from_node(viewport, input, style, &box_node.layout_id);

                let inline_ctx = InlineContext::new(containing_block);

                InlineLayout::layout(
                    &mut layout_tree.nodes,
                    input,
                    &inline_items,
                    &mut position_ctx,
                    inline_ctx,
                    &mut float_ctx,
                )
            }
            _ => {
                // FIXME: Should retain the old block context for the node being relayouted, so that it doesn't lose track of deferred positioned children.
                let mut block_ctx = BlockContext::default();

                let Some((node, size)) = BlockLayout::layout(
                    &mut layout_tree.nodes,
                    &box_node.layout_id,
                    &ComputedStyle::default(),
                    input,
                    &mut ctx,
                    &mut position_ctx,
                    &mut block_ctx,
                    &mut float_ctx,
                ) else {
                    return;
                };

                (vec![node], vec![Rect::new(0.0, 0.0, size.width, size.height)], size)
            }
        };

        if nodes.is_empty() {
            return;
        }

        let new_height = node_container.height;
        let delta = new_height - old_height;

        // TODO: Replace a range of nodes depends on inline and such.

        if delta.abs() < Self::EPSILON {
            return;
        }

        for ancestor_id in path.iter().skip(1) {
            let Some(mut node) = std::mem::take(&mut layout_tree.nodes[ancestor_id.index()]) else {
                panic!("Ancestor node not found in layout tree for layout_id: {:?}", ancestor_id);
            };

            let style = &*input.box_tree[ancestor_id].style;
            let prev_id = path[path.iter().position(|id| id == ancestor_id).unwrap() - 1];
            let changed_child_idx = node.children.iter().position(|child| *child == prev_id);

            if let Some(idx) = changed_child_idx {
                for sibling in &node.children[idx + 1..] {
                    Self::shift_y_recursively(&mut layout_tree.nodes, sibling, delta);
                }
            }

            if style.height.is_auto() {
                node.dimensions.height += delta;
            } else {
                break;
            }

            layout_tree.nodes[ancestor_id.index()] = Some(node);
        }

        layout_tree.content_height += delta;
    }

    fn shift_y_recursively(nodes: &mut Vec<Option<LayoutNode>>, id: &LayoutNodeId, delta: f64) {
        let Some(mut node) = std::mem::take(&mut nodes[id.index()]) else {
            panic!("Node not found in layout tree for layout_id: {:?}", id);
        };

        node.dimensions.y += delta;

        for child_id in &node.children {
            Self::shift_y_recursively(nodes, child_id, delta);
        }

        nodes[id.index()] = Some(node);
    }
}
