use crate::{
    LayoutNode, LayoutTree,
    context::{ImageContext, LayoutContext, PositionContext, TextContext},
    mode::{
        LayoutMode,
        block::{BlockContext, BlockLayout},
        inline::{InlineContext, InlineLayout}, //inline::{InlineContext, InlineLayout},
    },
    primitives::Rect,
};
use css_display::BoxTree;
use css_style::{ComputedStyle, StyleTree};
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
        let mut ctx = LayoutContext::new(viewport);
        let mut total_height = 0.0f64;
        let mut max_width = 0.0f64;
        let mut root_nodes = Vec::new();

        for layout_id in &input.box_tree.root_nodes {
            let mut block_ctx = BlockContext::default();

            let Some((node, size)) = BlockLayout::layout(
                layout_id,
                &ComputedStyle::default(),
                input,
                &mut ctx,
                &mut position_ctx,
                &mut block_ctx,
                false,
            ) else {
                continue;
            };

            total_height += size.height;
            max_width = max_width.max(size.width);

            root_nodes.push(node);
        }

        let mut tree = Self {
            root_nodes,
            content_height: total_height,
            content_width: max_width,
        };

        trace!("Initial layout complete, resolving deferred positions...");
        position_ctx.resolve_all(input, &mut tree);

        tree
    }

    /// Relayout a single node and its ancestors, updating the layout tree in place.
    ///
    /// # Panics
    /// * If the node or any of its ancestors are not found in the layout tree, which should never happen since the layout tree is built from the DOM tree.
    pub fn relayout_node<'css>(
        node_id: NodeId,
        viewport: Rect,
        layout_tree: &mut LayoutTree,
        style_tree: &'css StyleTree,
        input: &mut LayoutInput<'css>,
    ) {
        let node = &input.dom[node_id];

        let ancestors: Vec<NodeId> = input
            .dom
            .ancestors(node)
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
        let box_node = &input.box_tree[&old_layout.layout_id];

        let old_height = old_layout.dimensions.height;

        let mut position_ctx = PositionContext::new(viewport);
        let mut ctx = LayoutContext::new(old_layout.dimensions);

        let style = &style_tree[dirty_parent_id];
        let mode = LayoutMode::from(style);

        let (nodes, nodes_size) = match mode {
            LayoutMode::Inline => {
                let inline_items =
                    InlineLayout::collect_inline_items_from_nodes(viewport, input, style, &box_node.children);

                let inline_ctx = InlineContext::new(viewport);

                InlineLayout::layout(input, &inline_items, &mut ctx, &mut position_ctx, inline_ctx)
            }
            _ => {
                // FIXME: Should retain the old block context for the node being relayouted, so that it doesn't lose track of deferred positioned children.
                let mut block_ctx = BlockContext::default();

                let Some((node, size)) = BlockLayout::layout(
                    &box_node.layout_id,
                    &ComputedStyle::default(),
                    input,
                    &mut ctx,
                    &mut position_ctx,
                    &mut block_ctx,
                    false,
                ) else {
                    return;
                };

                (vec![node], Rect::new(0.0, 0.0, size.width, size.height))
            }
        };

        if nodes.is_empty() {
            return;
        }

        let new_height = nodes_size.height;
        let delta = new_height - old_height;

        // TODO: Replace a range of nodes depends on inline and such.
        *layout_tree.node_at_mut(&parent_path).unwrap() = nodes[0].clone();

        if delta.abs() < Self::EPSILON {
            return;
        }

        for ancestor_id in ancestors.iter().skip(1) {
            let Some(ancestor_path) = layout_tree.find_path(*ancestor_id) else {
                break;
            };
            let ancestor = layout_tree.node_at_mut(&ancestor_path).unwrap();
            let style = &style_tree[ancestor_id];

            let prev_id = ancestors[ancestors.iter().position(|id| id == ancestor_id).unwrap() - 1];

            let changed_child_idx = ancestor
                .children
                .iter()
                .position(|child| child.node_id == Some(prev_id));

            if let Some(idx) = changed_child_idx {
                for sibling in &mut ancestor.children[idx + 1..] {
                    Self::shift_y_recursively(sibling, delta);
                }
            }

            if style.height.is_auto() {
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
}
