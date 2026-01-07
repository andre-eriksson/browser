use css_cssom::CSSStyleSheet;
use html_dom::{DocumentRoot, NodeData, NodeId};

use crate::computed::ComputedStyle;

#[derive(Debug, Clone)]
pub struct StyledNode {
    pub node_id: NodeId,
    pub style: ComputedStyle,
    pub children: Vec<StyledNode>,
    pub text_content: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StyleTree {
    pub root_nodes: Vec<StyledNode>,
}

impl StyleTree {
    pub fn build(dom: &DocumentRoot, stylesheets: &[CSSStyleSheet]) -> Self {
        fn build_styled_node(
            node_id: NodeId,
            dom: &DocumentRoot,
            stylesheets: &[CSSStyleSheet],
            parent_style: Option<&ComputedStyle>,
        ) -> StyledNode {
            let computed_style = ComputedStyle::from_node(&node_id, dom, stylesheets, parent_style);

            let node = dom.get_node(&node_id).unwrap();

            let text_content = match &node.data {
                NodeData::Text(text) => Some(text.clone()),
                _ => None,
            };

            let children = node
                .children
                .iter()
                .map(|&child_id| {
                    build_styled_node(child_id, dom, stylesheets, Some(&computed_style))
                })
                .collect();

            StyledNode {
                node_id,
                style: computed_style,
                children,
                text_content,
            }
        }

        let root_nodes = dom
            .root_nodes
            .iter()
            .map(|&root_id| build_styled_node(root_id, dom, stylesheets, None))
            .collect();

        StyleTree { root_nodes }
    }
}
