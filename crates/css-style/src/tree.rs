//! This module defines the `StyledNode` and `StyleTree` structures, which represent the styled representation of the DOM tree. The `StyledNode`
//! structure contains the computed style for a single DOM node, while the `StyleTree` structure represents the entire styled tree corresponding to the DOM tree.
//! The `build` method of `StyleTree` constructs the styled tree from the given absolute context, DOM tree, and stylesheets by computing the styles for each node
//! based on the cascade rules and the provided stylesheets.

use std::sync::Arc;

use css_cssom::CSSStyleSheet;
use html_dom::{DocumentRoot, NodeData, NodeId, Tag};

use crate::cascade::{GeneratedRule, RuleIndex};
use crate::properties::AbsoluteContext;
use crate::{ComputedStyle, RelativeContext};

/// Represents a node in the style tree, which contains the computed style for a DOM node, its tag name (if it's an element), its children, and any text content (if it's a text node).
#[derive(Debug, Clone)]
pub struct StyledNode {
    pub node_id: NodeId,
    pub tag: Option<Tag>,
    pub style: ComputedStyle,
    pub children: Vec<StyledNode>,
    pub text_content: Option<String>,
}

impl StyledNode {
    /// Creates a new `StyledNode` with the given `node_id`. The `tag`, `style`, `children`, and `text_content` fields are initialized to their default values.
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

/// Represents the style tree, which is a hierarchical structure of styled nodes corresponding to the DOM tree. Each node in the style tree
/// contains the computed style for the corresponding DOM node, as well as its tag name (if it's an element), its children, and any text content (if it's a text node).
#[derive(Debug, Clone)]
pub struct StyleTree {
    /// The root nodes of the style tree.
    pub root_nodes: Vec<StyledNode>,
}

impl StyleTree {
    /// Builds the style tree from the given absolute context, DOM tree, and stylesheets. This function computes the styles for each node in the
    /// DOM tree based on the provided stylesheets and the cascade rules, and constructs the corresponding `StyledNode` for each DOM node.
    pub fn build(
        absolute_ctx: &AbsoluteContext,
        dom: &DocumentRoot,
        stylesheets: &[CSSStyleSheet],
    ) -> Self {
        let rules = GeneratedRule::build(stylesheets);
        let rule_index = RuleIndex::build(&rules);
        let mut relative_ctx = RelativeContext::default();

        fn build_styled_node(
            absolute_ctx: &AbsoluteContext,
            rel_ctx: &mut RelativeContext,
            node_id: NodeId,
            dom: &DocumentRoot,
            rules: &[GeneratedRule],
            rule_index: &RuleIndex,
            parent_style: Option<&ComputedStyle>,
        ) -> StyledNode {
            let computed_style = ComputedStyle::from_node(
                absolute_ctx,
                rel_ctx,
                &node_id,
                dom,
                rules,
                rule_index,
                parent_style,
            );

            rel_ctx.parent = Arc::new(computed_style.clone());

            let node = dom.get_node(&node_id).unwrap();

            let text_content = match &node.data {
                NodeData::Text(text) => Some(text.clone()),
                _ => None,
            };

            let saved_parent = Arc::clone(&rel_ctx.parent);

            let children = node
                .children
                .iter()
                .map(|&child_id| {
                    rel_ctx.parent = Arc::clone(&saved_parent);
                    build_styled_node(
                        absolute_ctx,
                        rel_ctx,
                        child_id,
                        dom,
                        rules,
                        rule_index,
                        Some(&computed_style),
                    )
                })
                .collect();

            rel_ctx.parent = saved_parent;

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
                build_styled_node(
                    absolute_ctx,
                    &mut relative_ctx,
                    root_id,
                    dom,
                    &rules,
                    &rule_index,
                    None,
                )
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
