use std::ops::Index;

use css_style::{ComputedStyle, StyleTree};
use css_values::text::Whitespace;
use html_dom::{DocumentRoot, NodeId};

use crate::node::{BoxNode, LayoutNodeId};

#[derive(Debug, Clone)]
pub struct BoxTree<'node> {
    pub root_nodes: Vec<LayoutNodeId>,
    pub nodes: Vec<BoxNode<'node>>,
}

impl<'node> BoxTree<'node> {
    pub fn new(dom: &'node DocumentRoot, style_tree: &'node StyleTree) -> Self {
        let mut tree = Self {
            root_nodes: Vec::with_capacity(dom.root_nodes.len()),
            nodes: Vec::with_capacity(dom.nodes.len()),
        };

        for node_id in &dom.root_nodes {
            tree.build_box_node(node_id, dom, style_tree, true);
        }

        tree
    }

    fn build_box_node(
        &mut self,
        node_id: &'node NodeId,
        dom: &'node DocumentRoot,
        style_tree: &'node StyleTree,
        is_root: bool,
    ) -> LayoutNodeId {
        let style = &style_tree[node_id];
        let layout_id = LayoutNodeId::new(self.nodes.len());

        if is_root {
            self.root_nodes.push(layout_id);
        }

        self.nodes
            .push(BoxNode::new_with_layout_id(layout_id, node_id, style, Vec::new()));

        let mut children = Vec::new();
        let mut inline_buffer = Vec::new();
        let mut saw_block = false;
        let all_block = dom[node_id]
            .children
            .iter()
            .filter(|child_id| !style_tree[*child_id].display.is_none())
            .all(|child_id| {
                let child_style = &style_tree[child_id];
                child_style.display.is_block() || Self::is_suppressable_whitespace(child_id, style, dom)
            });

        for child_id in &dom[node_id].children {
            let child_style = &style_tree[child_id];

            if child_style.display.is_none() {
                continue;
            }

            if all_block && Self::is_suppressable_whitespace(child_id, style, dom) {
                continue;
            }

            let child_layout_id = self.build_box_node(child_id, dom, style_tree, false);

            if child_style.display.is_inline() {
                inline_buffer.push(child_layout_id);
            } else if child_style.display.is_block() {
                if !inline_buffer.is_empty() {
                    let anon_layout_id = LayoutNodeId::new(self.nodes.len());
                    self.nodes
                        .push(BoxNode::new_anonymous_node_with_layout_id(anon_layout_id, inline_buffer, style));
                    children.push(anon_layout_id);
                    inline_buffer = Vec::new();
                }
                children.push(child_layout_id);
                saw_block = true;
            }
        }

        if !inline_buffer.is_empty() {
            if saw_block {
                let anon_layout_id = LayoutNodeId::new(self.nodes.len());
                self.nodes
                    .push(BoxNode::new_anonymous_node_with_layout_id(anon_layout_id, inline_buffer, style));
                children.push(anon_layout_id);
            } else {
                children.extend(inline_buffer);
            }
        }

        self.nodes[layout_id.index()].children = children;
        layout_id
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
        assert_eq!(tree.nodes[1].node_id, Some(NodeId(1)));
        assert_eq!(tree.nodes[2].node_id, Some(NodeId(2)));
        assert_eq!(tree.nodes[3].node_id, None);

        assert_eq!(tree.nodes[0].children, vec![LayoutNodeId::new(3), LayoutNodeId::new(2)]);
        assert_eq!(tree.nodes[3].children, vec![LayoutNodeId::new(1)]);
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
