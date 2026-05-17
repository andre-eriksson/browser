use css_style::StyleTree;
use html_dom::{DocumentRoot, NodeId};

use crate::node::BoxNode;

pub struct BoxTree {
    pub root_nodes: Vec<BoxNode>,
}

impl BoxTree {
    pub fn new(dom: &DocumentRoot, style_tree: &StyleTree) -> Self {
        let mut root_nodes = Vec::new();

        for node_id in &dom.root_nodes {
            let children = Self::build_box_node(node_id, dom, style_tree);
            root_nodes.push(BoxNode::new(node_id, children));
        }

        Self { root_nodes }
    }

    fn build_box_node(node_id: &NodeId, dom: &DocumentRoot, style_tree: &StyleTree) -> Vec<BoxNode> {
        let mut children = Vec::new();
        let mut inline_buffer = Vec::new();
        let mut saw_block = false;

        for child_id in &dom[node_id].children {
            let child_style = &style_tree[child_id];
            let childs_children = Self::build_box_node(child_id, dom, style_tree);

            if child_style.display.is_inline() {
                inline_buffer.push(BoxNode::new(child_id, childs_children));
            } else if child_style.display.is_block() {
                if !inline_buffer.is_empty() {
                    children.push(BoxNode::new_anonymous_node(inline_buffer));
                    inline_buffer = Vec::new();
                }
                children.push(BoxNode::new(child_id, childs_children));
                saw_block = true;
            }
        }

        if !inline_buffer.is_empty() {
            if saw_block {
                children.push(BoxNode::new_anonymous_node(inline_buffer));
            } else {
                children.extend(inline_buffer);
            }
        }

        children
    }
}
