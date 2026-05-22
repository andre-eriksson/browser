use css_display::{BoxNode, BoxTree};
use css_style::{ComputedStyle, StyleTree};
use html_dom::{DocumentRoot, NodeId};

use crate::{
    LayoutNode, LayoutTree,
    context::{ImageContext, LayoutContext, PositionContext, TextContext},
    mode::{
        LayoutMode,
        block::{BlockContext, BlockLayout},
        //inline::{InlineContext, InlineLayout},
    },
    primitives::{Rect, Size},
};

const EPSILON: f64 = 0.1;

/// Immutable references to the trees — passed everywhere, never mutated
pub struct LayoutInput<'a> {
    pub dom: &'a DocumentRoot,
    pub text: &'a mut TextContext,
}

impl LayoutTree {
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
        input: &mut LayoutInput,
        box_tree: &BoxTree,
        viewport: Rect,
        image_ctx: &ImageContext,
    ) -> Self {
        let mut position_ctx = PositionContext::new(viewport);
        let mut ctx = LayoutContext::new(viewport, image_ctx, &mut position_ctx);

        let mut total_height = 0.0f64;
        let mut max_width = 0.0f64;
        let mut root_nodes = Vec::new();

        for box_node in &box_tree.root_nodes {
            let Some((node, size)) = Self::layout_node(box_node, input, &ComputedStyle::default(), &mut ctx) else {
                continue;
            };

            total_height += node.dimensions.height;
            max_width = max_width.max(node.dimensions.width);

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

    /// Compute layout for a single node and its descendants
    pub(crate) fn layout_node(
        box_node: &BoxNode,
        input: &mut LayoutInput,
        parent_style: &ComputedStyle,
        ctx: &mut LayoutContext,
    ) -> Option<(LayoutNode, Size)> {
        let _layout_mode = LayoutMode::new(box_node);

        let mut block_ctx = BlockContext {
            cursor_y: ctx.containing_block().y,
            collapsed_margin: None,
            containing_width: ctx.containing_block().width,
        };

        BlockLayout::layout(box_node, parent_style, input, ctx, &mut block_ctx)
    }

    pub(crate) fn layout_nodes(
        box_nodes: &[BoxNode],
        parent_style: &ComputedStyle,
        input: &mut LayoutInput,
        containing_block: Rect,
        ctx: &mut LayoutContext,
    ) -> (Vec<LayoutNode>, Size) {
        let mode = if let Some(first_node) = box_nodes.first() {
            LayoutMode::new(first_node)
        } else {
            return (Vec::new(), Size::new(containing_block.width, containing_block.height));
        };

        match mode {
            LayoutMode::Block => {
                let mut layout_nodes = Vec::new();
                let mut current_y = containing_block.y;
                let mut block_ctx = BlockContext {
                    cursor_y: current_y,
                    collapsed_margin: None,
                    containing_width: containing_block.width,
                };

                for box_node in box_nodes {
                    let (mut layout_node, node_rect) =
                        BlockLayout::layout(box_node, parent_style, input, ctx, &mut block_ctx)
                            .unwrap_or_else(|| (LayoutNode::builder(box_node.node_id).build(), Size::new(0.0, 0.0)));

                    layout_node.dimensions.x = containing_block.x;
                    layout_node.dimensions.y = current_y;

                    current_y += layout_node.dimensions.height;
                    layout_nodes.push(layout_node);
                }

                let total_height = current_y - containing_block.y;
                let max_width = layout_nodes
                    .iter()
                    .map(|node| node.dimensions.width)
                    .fold(0.0, f64::max);

                (layout_nodes, Size::new(max_width, total_height))
            }
            LayoutMode::Inline => {
                // let inline_items = InlineLayout::collect_inline_items_from_nodes(
                //     containing_block,
                //     dom_tree,
                //     style_tree,
                //     parent_style,
                //     box_nodes,
                //     ctx.image_ctx(),
                // );
                // let inline_ctx = InlineContext::new(containing_block);

                // InlineLayout::layout(dom_tree, style_tree, &inline_items, ctx, text_ctx, inline_ctx)

                (Vec::new(), Size::new(0.0, 0.0))
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
        // let node = &dom_tree[node_id];

        // let ancestors: Vec<NodeId> = dom_tree.ancestors(node).into_iter().map(|n| n.id).collect();

        // let Some(&dirty_parent_id) = ancestors.first() else {
        //     return;
        // };
        // let Some(parent_path) = layout_tree.find_path(dirty_parent_id) else {
        //     return;
        // };

        // let old_layout = layout_tree.node_at(&parent_path).unwrap();
        // let old_height = old_layout.dimensions.height;

        // let mut position_ctx = PositionContext::new(viewport);
        // let mut ctx = LayoutContext::new(old_layout.dimensions, image_ctx, &mut position_ctx);
        // ctx.position_ctx().update_viewport(viewport);

        // let Some(mut new_node) = Self::layout_node(dom_tree, style_tree, &dirty_parent_id, &mut ctx, text_ctx) else {
        //     return;
        // };

        // Self::offset_children_y(&mut new_node.0.children, new_node.0.margin.top.to_px());

        // let new_height = new_node.0.dimensions.height;
        // let delta = new_height - old_height;

        // *layout_tree.node_at_mut(&parent_path).unwrap() = new_node.0;

        // if delta.abs() < EPSILON {
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

    fn shift_y_recursively(node: &mut LayoutNode, delta: f64) {
        node.dimensions.y += delta;
        for child in &mut node.children {
            Self::shift_y_recursively(child, delta);
        }
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

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use css_style::{ComputedStyle, Display};
    use css_values::display::{BoxDisplay, InsideDisplay, OutsideDisplay};
    use html_dom::{DomNode, Element, HtmlTag, NodeData, NodeId, Tag};

    use super::*;

    fn viewport() -> Rect {
        Rect::new(0.0, 0.0, 800.0, 600.0)
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
        let mut input = LayoutInput {
            dom: &dom_tree,
            text: &mut text_ctx,
        };
        let mut block_ctx = BlockContext {
            cursor_y: 0.0,
            collapsed_margin: None,
            containing_width: viewport().width,
        };

        let style = ComputedStyle::default();
        let box_node = BoxNode::new(&NodeId(0), &style, vec![]);
        let layout_node =
            BlockLayout::layout(&box_node, &ComputedStyle::default(), &mut input, &mut ctx, &mut block_ctx).unwrap();

        assert_eq!(layout_node.0.node_id, Some(NodeId(0)));
        assert_eq!(layout_node.0.dimensions.x, 0.0);
        assert_eq!(layout_node.0.dimensions.y, 0.0);
        assert_eq!(layout_node.0.dimensions.width, 800.0);
        assert_eq!(layout_node.0.dimensions.height, 0.0);
        assert_eq!(layout_node.0.children.len(), 0);
    }
}
