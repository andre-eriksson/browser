//! This module defines the `StyledNode` and `StyleTree` structures, which represent the styled representation of the DOM tree. The `StyledNode`
//! structure contains the computed style for a single DOM node, while the `StyleTree` structure represents the entire styled tree corresponding to the DOM tree.
//! The `build` method of `StyleTree` constructs the styled tree from the given absolute context, DOM tree, and stylesheets by computing the styles for each node
//! based on the cascade rules and the provided stylesheets.

use std::collections::HashMap;
use std::sync::Arc;

use browser_config::BrowserConfig;
use css_cssom::CSSStyleSheet;
use css_values::property::PropertyDescriptor;
use html_dom::{DocumentRoot, NodeId};

use crate::cascade::RuleIndex;
use crate::properties::AbsoluteContext;
use crate::rules::{GeneratedRule, Rules};
use crate::{ComputedStyle, RelativeContext};

#[derive(Debug, Default, Clone)]
pub struct PropertyRegistry {
    pub descriptors: HashMap<String, PropertyDescriptor>,
}

/// Represents a node in the style tree,
///
/// Contains the computed style for a DOM node, its tag name (if it's an element),
/// its children, any text content (if it's a text node), and its attributes (if it's an element).
#[derive(Debug, Clone)]
pub struct StyledNode {
    pub node_id: NodeId,
    pub style: ComputedStyle,
    pub children: Vec<Self>,
}

impl StyledNode {
    /// Creates a new `StyledNode` with the given `node_id`. The `tag`, `style`, `children`, and `text_content` fields are initialized to their default values.
    #[must_use]
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            style: ComputedStyle::default(),
            children: Vec::new(),
        }
    }
}

/// Represents the style tree,
///
/// It is a hierarchical structure of styled nodes corresponding to the DOM tree. Each node in the style tree
/// contains the computed style for the corresponding DOM node, as well as its tag name (if it's an element),
/// its children, and any text content (if it's a text node).
#[derive(Debug, Clone, Default)]
pub struct StyleTree {
    /// The root nodes of the style tree.
    pub root_nodes: Vec<StyledNode>,

    /// A registry of CSS properties, which contains the descriptors for all known CSS properties. This is used to validate and compute styles for each node in the style tree.
    pub property_registry: PropertyRegistry,
}

impl StyleTree {
    /// Builds the style tree from the given absolute context, DOM tree, and stylesheets. This function computes the styles for each node in the
    /// DOM tree based on the provided stylesheets and the cascade rules, and constructs the corresponding `StyledNode` for each DOM node.
    #[must_use]
    pub fn build(
        config: &BrowserConfig,
        absolute_ctx: &AbsoluteContext,
        dom: &DocumentRoot,
        stylesheets: &[CSSStyleSheet],
    ) -> Self {
        fn build_styled_node(
            config: &BrowserConfig,
            absolute_ctx: &AbsoluteContext,
            rel_ctx: &mut RelativeContext,
            node_id: NodeId,
            dom: &DocumentRoot,
            rules: &Rules,
            property_registry: &mut PropertyRegistry,
        ) -> StyledNode {
            let computed_style =
                ComputedStyle::from_node(config, absolute_ctx, rel_ctx, node_id, dom, rules, property_registry);

            rel_ctx.parent = Arc::new(computed_style.clone());

            let node = dom.get_node(&node_id).unwrap();
            let saved_parent = Arc::clone(&rel_ctx.parent);

            let children = node
                .children
                .iter()
                .map(|&child_id| {
                    rel_ctx.parent = Arc::clone(&saved_parent);
                    build_styled_node(config, absolute_ctx, rel_ctx, child_id, dom, rules, property_registry)
                })
                .collect();

            rel_ctx.parent = saved_parent;

            StyledNode {
                node_id,
                style: computed_style,
                children,
            }
        }

        let mut property_registry = PropertyRegistry::default();
        let rules = GeneratedRule::build(stylesheets, &mut property_registry, absolute_ctx);
        let rule_index = RuleIndex::build(&rules);
        let mut relative_ctx = RelativeContext::default();

        let root_nodes = dom
            .root_nodes
            .iter()
            .map(|&root_id| {
                build_styled_node(
                    config,
                    absolute_ctx,
                    &mut relative_ctx,
                    root_id,
                    dom,
                    &Rules {
                        generated: &rules,
                        index: &rule_index,
                    },
                    &mut property_registry,
                )
            })
            .collect();

        Self {
            root_nodes,
            property_registry,
        }
    }

    /// Finds a `StyledNode` in the style tree by its `NodeId`. This function performs a depth-first search through the style tree
    /// to find the node with the specified `NodeId`. If the node is found, it returns a reference to the `StyledNode`; otherwise,
    /// it returns `None`.
    #[must_use]
    pub fn find_node(&self, node_id: NodeId) -> Option<&StyledNode> {
        fn find_in_node(node: &StyledNode, node_id: NodeId) -> Option<&StyledNode> {
            if node.node_id == node_id {
                return Some(node);
            }
            for child in &node.children {
                if let Some(found) = find_in_node(child, node_id) {
                    return Some(found);
                }
            }
            None
        }

        for root in &self.root_nodes {
            if let Some(found) = find_in_node(root, node_id) {
                return Some(found);
            }
        }
        None
    }
}

impl From<StyledNode> for StyleTree {
    fn from(value: StyledNode) -> Self {
        Self {
            root_nodes: vec![value],
            property_registry: PropertyRegistry::default(),
        }
    }
}
