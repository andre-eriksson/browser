use std::{fmt::Debug, ops::Index};

use css_style::{ComputedStyle, StyleTree};
use css_values::text::Whitespace;
use html_dom::{DocumentRoot, NodeId};

use crate::node::{BoxNode, LayoutNodeId};

#[derive(Debug, Clone)]
pub struct BoxTree<'node> {
    pub root_nodes: Vec<LayoutNodeId>,
    pub nodes: Vec<BoxNode<'node>>,
    pub dom_to_layout: Vec<Option<LayoutNodeId>>,
}

impl<'node> BoxTree<'node> {
    pub fn new(dom: &'node DocumentRoot, style_tree: &'node StyleTree) -> Self {
        let mut root_nodes = Vec::with_capacity(dom.root_nodes.len());
        let mut nodes = Vec::with_capacity(dom.nodes.len());
        let mut dom_to_layout = vec![None; dom.nodes.len()];

        for node_id in &dom.root_nodes {
            if let Some(id) = Self::build_box_node(None, node_id, dom, style_tree, &mut nodes, &mut dom_to_layout) {
                root_nodes.push(id);

                if dom.get_node(node_id).is_some() {
                    dom_to_layout[node_id.index()] = Some(id);
                }
            }
        }

        debug_assert!(nodes.is_sorted_by(|a, b| a.layout_id.index() < b.layout_id.index()));

        Self {
            root_nodes,
            nodes,
            dom_to_layout,
        }
    }

    fn build_box_node(
        parent_id: Option<LayoutNodeId>,
        node_id: &'node NodeId,
        dom: &'node DocumentRoot,
        style_tree: &'node StyleTree,
        nodes: &mut Vec<BoxNode<'node>>,
        dom_to_layout: &mut Vec<Option<LayoutNodeId>>,
    ) -> Option<LayoutNodeId> {
        let style = &style_tree[node_id];

        if style.display.is_none() {
            return None;
        }

        if Self::is_suppressable_whitespace(node_id, style, dom) {
            return None;
        }

        let layout_id = LayoutNodeId::new(nodes.len());
        nodes.push(BoxNode::new(parent_id, layout_id, node_id, style, Vec::new()));

        if dom.get_node(node_id).is_some() {
            dom_to_layout[node_id.index()] = Some(layout_id);
        }

        let (all_block, all_inline) = dom[node_id]
            .children
            .iter()
            .filter(|child_id| !style_tree[*child_id].display.is_none())
            .fold((true, true), |(all_block, all_inline), child_id| {
                let child_style = &style_tree[child_id];
                (all_block && child_style.display.is_block(), all_inline && child_style.display.is_inline())
            });
        let needs_anonymous_wrapping = !all_block && !all_inline;

        let mut layout_children: Vec<LayoutNodeId> = Vec::new();
        let mut anon_children: Vec<LayoutNodeId> = Vec::new();
        let mut current_anon_id: Option<LayoutNodeId> = None;

        for child_id in &dom[node_id].children {
            let child_style = &style_tree[child_id];

            if child_style.display.is_none() || Self::is_suppressable_whitespace(child_id, style, dom) {
                continue;
            }

            if needs_anonymous_wrapping && child_style.display.is_inline() {
                if current_anon_id.is_none() {
                    let anon_id = LayoutNodeId::new(nodes.len());
                    nodes.push(BoxNode::new_anonymous_node(Some(layout_id), anon_id, style, Vec::new()));
                    layout_children.push(anon_id);
                    current_anon_id = Some(anon_id);
                }

                let anon_id = current_anon_id.unwrap();
                if let Some(child_layout_id) =
                    Self::build_box_node(Some(anon_id), child_id, dom, style_tree, nodes, dom_to_layout)
                {
                    anon_children.push(child_layout_id);
                }
            } else {
                if let Some(anon_id) = current_anon_id.take() {
                    nodes[anon_id.index()].children = std::mem::take(&mut anon_children);
                }

                if let Some(child_layout_id) =
                    Self::build_box_node(Some(layout_id), child_id, dom, style_tree, nodes, dom_to_layout)
                {
                    layout_children.push(child_layout_id);
                }
            }
        }

        if let Some(anon_id) = current_anon_id.take() {
            nodes[anon_id.index()].children = std::mem::take(&mut anon_children);
        }

        nodes[layout_id.index()].children = layout_children;
        Some(layout_id)
    }

    fn is_suppressable_whitespace(node_id: &NodeId, parent_style: &ComputedStyle, dom: &DocumentRoot) -> bool {
        let node = &dom[node_id];

        if let Some(text) = node.data.as_text() {
            let is_all_whitespace = text.chars().all(|c| c.is_ascii_whitespace());
            let collapses =
                matches!(parent_style.whitespace, Whitespace::Normal | Whitespace::Nowrap | Whitespace::PreLine);
            is_all_whitespace && collapses
        } else {
            false
        }
    }

    #[must_use]
    pub fn ancestors(&'node self, node: &BoxNode) -> Vec<&'node BoxNode<'node>> {
        let mut result = Vec::new();
        let mut current = node.parent_id;

        while let Some(pid) = current {
            let parent_node = &self[pid];

            result.push(parent_node);
            current = parent_node.parent_id;
        }

        result
    }
}

impl<'node> Index<&LayoutNodeId> for BoxTree<'node> {
    type Output = BoxNode<'node>;

    fn index(&self, layout_id: &LayoutNodeId) -> &Self::Output {
        &self.nodes[layout_id.index()]
    }
}

impl<'node> Index<LayoutNodeId> for BoxTree<'node> {
    type Output = BoxNode<'node>;

    fn index(&self, layout_id: LayoutNodeId) -> &Self::Output {
        &self.nodes[layout_id.index()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use css_style::{ComputedStyle, StyleTree};
    use css_values::display::OutsideDisplay;
    use html_dom::{DocumentRoot, DomNode, Element, NodeData, NodeId, Tag};
    use std::collections::{HashMap, HashSet};

    fn element_node(id: usize, parent: Option<usize>, children: Vec<usize>) -> DomNode {
        DomNode {
            id: NodeId(id),
            parent: parent.map(NodeId),
            children: children.into_iter().map(NodeId).collect(),
            data: NodeData::Element(Element::new(Tag::Unknown(format!("node-{id}")), HashSet::new(), HashMap::new())),
        }
    }

    fn build_dom() -> DocumentRoot {
        DocumentRoot {
            nodes: vec![
                element_node(0, None, vec![1, 2]),
                element_node(1, Some(0), vec![]),
                element_node(2, Some(0), vec![]),
            ],
            root_nodes: vec![NodeId(0)],
        }
    }

    fn build_styles() -> StyleTree {
        let root = ComputedStyle {
            display: OutsideDisplay::Block.into(),
            ..Default::default()
        };

        let inline_child = ComputedStyle::default();

        let block_child = ComputedStyle {
            display: OutsideDisplay::Block.into(),
            ..Default::default()
        };

        StyleTree::from(vec![root, inline_child, block_child])
    }

    #[test]
    fn flat_tree_keeps_nodes_in_insertion_order() {
        let dom = build_dom();
        let styles = build_styles();
        let tree = BoxTree::new(&dom, &styles);

        assert_eq!(tree.nodes.len(), 4);
        assert!(
            tree.nodes
                .iter()
                .enumerate()
                .all(|(index, node)| node.layout_id.index() == index)
        );

        assert_eq!(tree.nodes[0].node_id, Some(NodeId(0)));
        assert_eq!(tree.nodes[1].node_id, None);
        assert_eq!(tree.nodes[2].node_id, Some(NodeId(1)));
        assert_eq!(tree.nodes[3].node_id, Some(NodeId(2)));

        assert_eq!(tree.nodes[0].children, vec![LayoutNodeId::new(1), LayoutNodeId::new(3)]);
        assert_eq!(tree.nodes[1].children, vec![LayoutNodeId::new(2)]);
    }

    #[test]
    fn anonymous_box_is_indexable_by_layout_id() {
        let dom = build_dom();
        let styles = build_styles();
        let tree = BoxTree::new(&dom, &styles);
        let anonymous_id = tree.nodes[0].children[0];

        assert!(tree[anonymous_id].node_id.is_none());
        assert!(tree[&anonymous_id].node_id.is_none());
        assert_eq!(tree[anonymous_id].layout_id, anonymous_id);
        assert!(tree[anonymous_id].style.display.is_block());
        assert_eq!(tree[anonymous_id].children, vec![LayoutNodeId::new(1)]);
    }
}
