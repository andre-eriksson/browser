use css_cssom::CSSStyleSheet;
use html_dom::{DocumentRoot, NodeId};

use crate::computed::ComputedStyle;

#[derive(Debug)]
pub struct StyledNode {
    pub node_id: NodeId,
    pub style: ComputedStyle,
    pub children: Vec<StyledNode>,
}

#[derive(Debug)]
pub struct StyleTree {
    pub root_nodes: Vec<StyledNode>,
}

impl StyleTree {
    pub fn build(dom: &DocumentRoot, stylesheets: &[CSSStyleSheet]) -> Self {
        fn build_styled_node(
            node_id: NodeId,
            dom: &DocumentRoot,
            stylesheets: &[CSSStyleSheet],
        ) -> StyledNode {
            let computed_style = ComputedStyle::from_node(&node_id, dom, stylesheets);

            let node = dom.get_node(&node_id).unwrap();
            let children = node
                .children
                .iter()
                .map(|&child_id| build_styled_node(child_id, dom, stylesheets))
                .collect();

            StyledNode {
                node_id,
                style: computed_style,
                children,
            }
        }

        let root_nodes = dom
            .root_nodes
            .iter()
            .map(|&root_id| build_styled_node(root_id, dom, stylesheets))
            .collect();

        StyleTree { root_nodes }
    }
}
