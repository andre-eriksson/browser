use css_display::BoxTree;
use css_style::ComputedStyle;
use html_dom::{DocumentRoot, NodeId};

use crate::{
    LayoutTree,
    context::{ImageContext, LayoutContext, TextContext},
    mode::{
        block::{BlockContext, BlockLayout}, //inline::{InlineContext, InlineLayout},
    },
    primitives::Rect,
};

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
        // let mut position_ctx = PositionContext::new(viewport);
        let mut ctx = LayoutContext::new(viewport);

        let mut total_height = 0.0f64;
        let mut max_width = 0.0f64;
        let mut root_nodes = Vec::new();

        for layout_id in &input.box_tree.root_nodes {
            let mut block_ctx = BlockContext::default();

            let Some((node, size)) =
                BlockLayout::layout(layout_id, &ComputedStyle::default(), input, &mut ctx, &mut block_ctx, false)
            else {
                continue;
            };

            total_height += size.height;
            max_width = max_width.max(size.width);

            root_nodes.push(node);
        }

        // for defered_node in ctx.position_ctx().resolve_all(input, image_ctx) {
        //     // Self::offset_children_y(&mut defered_node.0.children, defered_node.0.margin.top.to_px());

        //     total_height = total_height.max(defered_node.1.height);
        //     max_width = max_width.max(defered_node.1.width);

        //     root_nodes.push(defered_node.0);
        // }

        Self {
            root_nodes,
            content_height: total_height,
            content_width: max_width,
        }
    }

    /// Relayout a single node and its ancestors, updating the layout tree in place.
    ///
    /// # Panics
    /// * If the node or any of its ancestors are not found in the layout tree, which should never happen since the layout tree is built from the DOM tree.
    pub fn relayout_node(_node_id: NodeId, _viewport: Rect, _layout_tree: &mut LayoutTree, _input: &mut LayoutInput) {
        // let node = &input.dom[node_id];

        // let ancestors: Vec<NodeId> = input
        //     .dom
        //     .ancestors(node)
        //     .into_iter()
        //     .map(|n| n.id)
        //     .collect();

        // let Some(&dirty_parent_id) = ancestors.first() else {
        //     return;
        // };
        // let Some(parent_path) = layout_tree.find_path(dirty_parent_id) else {
        //     return;
        // };

        // let old_layout = layout_tree.node_at(&parent_path).unwrap();
        // let old_box_node = box_tree[dirty_parent_id];

        // let old_height = old_layout.dimensions.height;

        // // let mut position_ctx = PositionContext::new(viewport);
        // let mut ctx = LayoutContext::new(old_layout.dimensions);
        // // ctx.position_ctx().update_viewport(viewport);

        // let mode = LayoutMode::new(box_node)

        // let Some(mut new_node) = Self::layout_node(dom_tree, style_tree, &dirty_parent_id, &mut ctx, text_ctx) else {
        //     return;
        // };

        // let new_height = new_node.0.dimensions.height;
        // let delta = new_height - old_height;

        // *layout_tree.node_at_mut(&parent_path).unwrap() = new_node.0;

        // if delta.abs() < Self::EPSILON {
        //     return;
        // }

        // for ancestor_id in ancestors.iter().skip(1) {
        //     let Some(ancestor_path) = layout_tree.find_path(*ancestor_id) else {
        //         break;
        //     };
        //     let ancestor = layout_tree.node_at_mut(&ancestor_path).unwrap();

        //     let prev_id = ancestors[ancestors.iter().position(|id| id == ancestor_id).unwrap() - 1];

        //     let changed_child_idx = ancestor
        //         .children
        //         .iter()
        //         .position(|child| child.node_id == prev_id);

        //     if let Some(idx) = changed_child_idx {
        //         for sibling in &mut ancestor.children[idx + 1..] {
        //             Self::shift_y_recursively(sibling, delta);
        //         }
        //     }

        //     if ancestor.is_height_auto {
        //         ancestor.dimensions.height += delta;
        //     } else {
        //         break;
        //     }
        // }

        // layout_tree.content_height += delta;
    }

    // pub(crate) fn collect_children<F>(
    //     ctx: &mut LayoutContext,
    //     dom_tree: &DocumentRoot,
    //     style_tree: &StyleTree,
    //     parent_id: &NodeId,
    //     start_idx: &mut usize,
    //     condition: F,
    // ) -> Vec<NodeId>
    // where
    //     F: Fn(&NodeId) -> bool,
    // {
    //     let mut collected = Vec::new();
    //     let parent_node = &dom_tree[parent_id];
    //     let children = &parent_node.children;

    //     for child in children.iter().skip(*start_idx) {
    //         let style = &style_tree[child];

    //         if style.position.is_out_of_flow() && !ctx.is_deferred() {
    //             let containing_block = if style.position == Position::Fixed {
    //                 ctx.containing_block()
    //             } else {
    //                 ctx.positioned_containing_block()
    //             };

    //             ctx.position_ctx().defer(child, containing_block);
    //             *start_idx += 1;
    //             continue;
    //         }

    //         if condition(child) {
    //             collected.push(*child);
    //             *start_idx += 1;
    //         } else {
    //             break;
    //         }
    //     }

    //     collected
    // }
}
