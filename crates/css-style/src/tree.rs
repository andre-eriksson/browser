use css_cssom::CSSStyleSheet;
use html_dom::{DocumentRoot, NodeData, NodeId, Tag};

use crate::cascade::GeneratedRule;
use crate::properties::AbsoluteContext;
use crate::{ComputedStyle, RelativeContext};

#[derive(Debug, Clone)]
pub struct StyledNode {
    pub node_id: NodeId,
    pub tag: Option<Tag>,
    pub style: ComputedStyle,
    pub children: Vec<StyledNode>,
    pub text_content: Option<String>,
}

impl StyledNode {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            tag: None,
            style: ComputedStyle::default(),
            children: Vec::new(),
            text_content: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StyleTree {
    pub root_nodes: Vec<StyledNode>,
}

impl StyleTree {
    pub fn build(
        absolute_ctx: &AbsoluteContext,
        dom: &DocumentRoot,
        stylesheets: &[CSSStyleSheet],
    ) -> Self {
        let rules = GeneratedRule::build(stylesheets);
        let mut relative_ctx = RelativeContext::default();

        fn build_styled_node(
            absolute_ctx: &AbsoluteContext,
            rel_ctx: &mut RelativeContext,
            node_id: NodeId,
            dom: &DocumentRoot,
            rules: &[GeneratedRule],
            parent_style: Option<&ComputedStyle>,
        ) -> StyledNode {
            let computed_style =
                ComputedStyle::from_node(absolute_ctx, rel_ctx, &node_id, dom, rules, parent_style);

            let node = dom.get_node(&node_id).unwrap();

            let text_content = match &node.data {
                NodeData::Text(text) => Some(text.clone()),
                _ => None,
            };

            let children = node
                .children
                .iter()
                .map(|&child_id| {
                    build_styled_node(
                        absolute_ctx,
                        rel_ctx,
                        child_id,
                        dom,
                        rules,
                        Some(&computed_style),
                    )
                })
                .collect();

            StyledNode {
                node_id,
                tag: node.data.as_element().map(|e| e.tag.clone()),
                style: computed_style,
                children,
                text_content,
            }
        }

        let root_nodes = dom
            .root_nodes
            .iter()
            .map(|&root_id| {
                build_styled_node(absolute_ctx, &mut relative_ctx, root_id, dom, &rules, None)
            })
            .collect();

        StyleTree { root_nodes }
    }
}

impl From<StyledNode> for StyleTree {
    fn from(value: StyledNode) -> Self {
        Self {
            root_nodes: vec![value],
        }
    }
}
