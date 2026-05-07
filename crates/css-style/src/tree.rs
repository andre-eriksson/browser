//! This module defines the `StyledNode` and `StyleTree` structures, which represent the styled representation of the DOM tree. The `StyledNode`
//! structure contains the computed style for a single DOM node, while the `StyleTree` structure represents the entire styled tree corresponding to the DOM tree.
//! The `build` method of `StyleTree` constructs the styled tree from the given absolute context, DOM tree, and stylesheets by computing the styles for each node
//! based on the cascade rules and the provided stylesheets.

use std::collections::HashMap;

use browser_config::BrowserConfig;
use css_cssom::CSSStyleSheet;
use css_values::property::PropertyDescriptor;
use html_dom::DocumentRoot;

use crate::ComputedStyle;
use crate::cascade::RuleIndex;
use crate::properties::AbsoluteContext;
use crate::rules::{GeneratedRule, Rules};

/// Represents the property registry, for storing the descriptors of CSS properties, parsed via the @property rule in the stylesheets.
#[derive(Debug, Default, Clone)]
pub struct PropertyRegistry {
    pub descriptors: HashMap<String, PropertyDescriptor>,
}

/// Represents the style tree,
///
/// It is a hierarchical structure of styled nodes corresponding to the DOM tree. Each node in the style tree
/// contains the computed style for the corresponding DOM node, as well as its tag name (if it's an element),
/// its children, and any text content (if it's a text node).
#[derive(Debug, Clone, Default)]
pub struct StyleTree {
    /// The styled nodes corresponding to the DOM nodes. Accessed via the `NodeId` as the index.
    pub nodes: Vec<ComputedStyle>,

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
        let mut property_registry = PropertyRegistry::default();
        let rules = GeneratedRule::build(stylesheets, &mut property_registry, absolute_ctx);
        let rule_index = RuleIndex::build(&rules);

        let mut nodes = Vec::with_capacity(dom.nodes.len());

        for node in &dom.nodes {
            let computed_style = ComputedStyle::from_node(
                config,
                absolute_ctx,
                node.id,
                dom,
                &Rules {
                    generated: &rules,
                    index: &rule_index,
                },
                &mut property_registry,
                &nodes,
            );

            nodes.push(computed_style);
        }

        Self {
            nodes,
            property_registry,
        }
    }
}
