use css_style::{ComputedStyle, StyleTree};
use css_values::text::Whitespace;
use html_dom::{DocumentRoot, NodeId};

use crate::node::BoxNode;

#[derive(Debug, Clone)]
pub struct BoxTree<'node> {
    pub root_nodes: Vec<BoxNode<'node>>,
}

impl<'node> BoxTree<'node> {
    pub fn new(dom: &'node DocumentRoot, style_tree: &'node StyleTree) -> Self {
        let mut root_nodes = Vec::new();

        for node_id in &dom.root_nodes {
            let children = Self::build_box_node(node_id, dom, style_tree);

            let style = &style_tree[node_id];
            root_nodes.push(BoxNode::new(node_id, style, children));
        }

        Self { root_nodes }
    }

    fn build_box_node(node_id: &NodeId, dom: &'node DocumentRoot, style_tree: &'node StyleTree) -> Vec<BoxNode<'node>> {
        let style = &style_tree[node_id];
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

            let childs_children = Self::build_box_node(child_id, dom, style_tree);

            if child_style.display.is_inline() {
                inline_buffer.push(BoxNode::new(child_id, child_style, childs_children));
            } else if child_style.display.is_block() {
                if !inline_buffer.is_empty() {
                    children.push(BoxNode::new_anonymous_node(inline_buffer, style));
                    inline_buffer = Vec::new();
                }
                children.push(BoxNode::new(child_id, child_style, childs_children));
                saw_block = true;
            }
        }

        if !inline_buffer.is_empty() {
            if saw_block {
                children.push(BoxNode::new_anonymous_node(inline_buffer, style));
            } else {
                children.extend(inline_buffer);
            }
        }

        children
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
