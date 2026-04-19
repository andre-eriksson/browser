use std::collections::HashSet;

use css_cssom::{CssToken, CssTokenKind, HashType};
use html_dom::{DocumentRoot, DomNode, Element, HtmlTag, Tag};

use crate::{
    parser::CaseSensitivity,
    selector::{CompoundSelector, CompoundSelectorSequence},
};

/// The operators used in attribute selectors
#[derive(Debug)]
pub enum AttributeOperator {
    /// The '=' operator, which matches if the attribute value is exactly the specified value
    Equals,

    /// The '~=' operator, which matches if the attribute value is a whitespace-separated list of words,
    /// one of which is exactly the specified value
    Includes,

    /// The '|=' operator, which matches if the attribute value is exactly the specified value
    /// or starts with the specified value followed by a hyphen
    DashMatch,

    /// The '^=' operator, which matches if the attribute value starts with the specified value
    PrefixMatch,

    /// The '$=' operator, which matches if the attribute value ends with the specified value
    SuffixMatch,

    /// The '*=' operator, which matches if the attribute value contains the specified value
    SubstringMatch,
}

/// The combinators used in CSS selectors
#[derive(Debug)]
pub enum Combinator {
    /// The descendant combinator (a space)
    /// Any ancestor relationship
    Descendant,

    /// The child combinator ('>')
    /// Direct parent-child relationship
    Child,

    /// The adjacent sibling combinator ('+')
    /// Directly following sibling relationship
    AdjacentSibling,

    /// The general sibling combinator ('~')
    /// Any following sibling relationship
    GeneralSibling,
}

/// Check if an element matches a list of compound selectors
///
/// # Arguments
/// * `compound_selectors` - A slice of CompoundSelector representing the selector
/// * `element` - The DOM element to check for a match
///
/// # Returns
/// * `bool` - True if the element matches the compound selectors, false otherwise
fn matches_compound_selectors(
    compound_selectors: &[CompoundSelector],
    element: &Element,
    class_set: Option<&HashSet<String>>,
    tree: &DocumentRoot,
    node: &DomNode,
) -> bool {
    for compound_selector in compound_selectors {
        if !matches_simple_selectors(&compound_selector.tokens, element, class_set) {
            return false;
        }

        for is_selector_list in &compound_selector.is_selector_lists {
            let any_match = is_selector_list
                .iter()
                .any(|sequence| matches_compound(sequence, tree, node, class_set));

            if !any_match {
                return false;
            }
        }

        for attribute_selector in &compound_selector.attribute_selectors {
            let attr_name = match &attribute_selector.attribute {
                Some(token) => {
                    if let CssTokenKind::Ident(name) = &token.kind {
                        name
                    } else {
                        return false;
                    }
                }
                None => return false,
            };

            let operator = match &attribute_selector.operator {
                Some(op) => op,
                None => {
                    if !element.has_attribute(attr_name) {
                        return false;
                    } else {
                        continue;
                    }
                }
            };

            let element_attribute_value = match element.get_attribute(attr_name) {
                Some(value) => value,
                None => return false,
            };

            let expected_value = match &attribute_selector.value {
                Some(token) => {
                    if let CssTokenKind::Ident(val) | CssTokenKind::String(val) = &token.kind {
                        val
                    } else {
                        return false;
                    }
                }
                None => return false,
            };

            let default_sensitivity = CaseSensitivity::default();
            let sensitivity = attribute_selector
                .case
                .as_ref()
                .unwrap_or(&default_sensitivity);

            match operator {
                AttributeOperator::Equals => match sensitivity {
                    CaseSensitivity::CaseInsensitive => {
                        if !element_attribute_value.eq_ignore_ascii_case(expected_value) {
                            return false;
                        }
                    }
                    CaseSensitivity::CaseSensitive => {
                        if element_attribute_value != *expected_value {
                            return false;
                        }
                    }
                },
                AttributeOperator::Includes => match sensitivity {
                    CaseSensitivity::CaseInsensitive => {
                        if !element_attribute_value
                            .split_whitespace()
                            .any(|word| word.eq_ignore_ascii_case(expected_value))
                        {
                            return false;
                        }
                    }
                    CaseSensitivity::CaseSensitive => {
                        if !element_attribute_value
                            .split_whitespace()
                            .any(|word| word == expected_value.as_str())
                        {
                            return false;
                        }
                    }
                },
                AttributeOperator::DashMatch => match sensitivity {
                    CaseSensitivity::CaseInsensitive => {
                        if !(element_attribute_value.eq_ignore_ascii_case(expected_value)
                            || element_attribute_value.len() > expected_value.len()
                                && element_attribute_value.as_bytes()[expected_value.len()] == b'-'
                                && element_attribute_value.as_bytes()[..expected_value.len()]
                                    .eq_ignore_ascii_case(expected_value.as_bytes()))
                        {
                            return false;
                        }
                    }
                    CaseSensitivity::CaseSensitive => {
                        if !(element_attribute_value.eq_ignore_ascii_case(expected_value)
                            || element_attribute_value.starts_with(expected_value)
                                && element_attribute_value.as_bytes().get(expected_value.len()) == Some(&b'-'))
                        {
                            return false;
                        }
                    }
                },
                AttributeOperator::PrefixMatch => match sensitivity {
                    CaseSensitivity::CaseInsensitive => {
                        if !element_attribute_value
                            .as_bytes()
                            .get(..expected_value.len())
                            .is_some_and(|s| s.eq_ignore_ascii_case(expected_value.as_bytes()))
                        {
                            return false;
                        }
                    }
                    CaseSensitivity::CaseSensitive => {
                        if !element_attribute_value.starts_with(expected_value) {
                            return false;
                        }
                    }
                },
                AttributeOperator::SuffixMatch => match sensitivity {
                    CaseSensitivity::CaseInsensitive => {
                        let len = element_attribute_value.len();
                        let sub_len = expected_value.len();

                        if sub_len > len
                            || !element_attribute_value.as_bytes()[len - sub_len..]
                                .eq_ignore_ascii_case(expected_value.as_bytes())
                        {
                            return false;
                        }
                    }
                    CaseSensitivity::CaseSensitive => {
                        if !element_attribute_value.ends_with(expected_value) {
                            return false;
                        }
                    }
                },
                AttributeOperator::SubstringMatch => match sensitivity {
                    CaseSensitivity::CaseInsensitive => {
                        if !element_attribute_value
                            .as_bytes()
                            .windows(expected_value.len())
                            .any(|window| window.eq_ignore_ascii_case(expected_value.as_bytes()))
                        {
                            return false;
                        }
                    }
                    CaseSensitivity::CaseSensitive => {
                        if !element_attribute_value.contains(expected_value) {
                            return false;
                        }
                    }
                },
            }
        }
    }

    true
}

/// Check if an element matches a list of simple selectors
///
/// # Arguments
/// * `simple_selectors` - A slice of CSS tokens representing simple selectors
/// * `element` - The DOM element to check for a match
///
/// # Returns
/// * `bool` - True if the element matches the simple selectors, false otherwise
fn matches_simple_selectors(
    simple_selectors: &[CssToken],
    element: &Element,
    class_set: Option<&HashSet<String>>,
) -> bool {
    for i in 0..simple_selectors.len() {
        let previous_token = &simple_selectors.get(i.wrapping_sub(1));
        let current_token = &simple_selectors[i];
        let next_token = simple_selectors.get(i + 1);

        match &current_token.kind {
            CssTokenKind::Ident(ident) => {
                let prev = previous_token.map(|t| &t.kind);
                let next = next_token.map(|t| &t.kind);

                if prev.is_none() || matches!(prev, Some(CssTokenKind::Whitespace)) {
                    match next {
                        None | Some(CssTokenKind::Delim(_)) | Some(CssTokenKind::Whitespace)
                            if Tag::from_str_insensitive(ident) != element.tag && ident != "*" =>
                        {
                            return false;
                        }
                        _ => {}
                    }
                } else if let Some(CssTokenKind::Delim(delim)) = prev {
                    let Some(class_set) = class_set else {
                        return false;
                    };

                    if *delim == '.' && !class_set.contains(ident) {
                        return false;
                    }
                } else if matches!(prev, Some(CssTokenKind::Colon)) {
                    return false;
                }
            }
            CssTokenKind::Hash { value, type_flag } => {
                if *type_flag != HashType::Id {
                    continue;
                }

                let id = if let Some(element_id) = element.id() {
                    element_id
                } else {
                    return false;
                };

                if !id.eq_ignore_ascii_case(value) {
                    return false;
                }
            }
            CssTokenKind::Colon => {
                let _prev = previous_token.map(|t| &t.kind);
                let next = next_token.map(|t| &t.kind);

                if let Some(CssTokenKind::Ident(ident)) = next {
                    if ident.eq_ignore_ascii_case("root") {
                        return element.tag == Tag::Html(HtmlTag::Html);
                    } else if ident.eq_ignore_ascii_case("link") {
                        return element.tag == Tag::Html(HtmlTag::A) && element.has_attribute("href");
                    }
                }

                // TODO: Handle more pseudo-classes and pseudo-elements

                return false;
            }
            _ => continue, // TODO: Handle other simple selectors
        }
    }

    true
}

/// Check if a DOM node matches a sequence of compound selectors
///
/// # Arguments
/// * `sequence` - A vector of CompoundSelectorSequence representing the selector
/// * `tree` - The DocumentRoot representing the DOM tree
/// * `node` - The DomNode to check for a match
///
/// # Returns
/// * `bool` - True if the node matches the selector sequence, false otherwise
pub fn matches_compound(
    sequence: &[CompoundSelectorSequence],
    tree: &DocumentRoot,
    node: &DomNode,
    class_set: Option<&HashSet<String>>,
) -> bool {
    fn matches_from_index(
        sequences: &[CompoundSelectorSequence],
        index: usize,
        tree: &DocumentRoot,
        node: &DomNode,
        class_set: Option<&HashSet<String>>,
    ) -> bool {
        if sequences.is_empty() {
            return false;
        }

        let element = match node.data.as_element() {
            Some(elem) => elem,
            None => return false,
        };

        let current = &sequences[index];
        if !matches_compound_selectors(&current.compound_selectors, element, class_set, tree, node) {
            return false;
        }

        if index == 0 {
            return true;
        }

        let relation = match &sequences[index - 1].combinator {
            Some(rel) => rel,
            None => return false,
        };

        let parent_id = match node.parent {
            Some(parent_id) => parent_id,
            None => return false,
        };

        let parent_node = match tree.get_node(&parent_id) {
            Some(node) => node,
            None => return false,
        };

        match relation {
            Combinator::Child => {
                let parent_element = match parent_node.data.as_element() {
                    Some(elem) => elem,
                    None => return false,
                };

                let class_set = parent_element.class_set.as_ref();
                matches_from_index(sequences, index - 1, tree, parent_node, class_set)
            }
            Combinator::Descendant => {
                let mut ancestor = Some(parent_node);

                while let Some(candidate) = ancestor {
                    if let Some(candidate_element) = candidate.data.as_element() {
                        let class_set = candidate_element.class_set.as_ref();
                        if matches_from_index(sequences, index - 1, tree, candidate, class_set) {
                            return true;
                        }
                    }

                    ancestor = candidate.parent.and_then(|id| tree.get_node(&id));
                }

                false
            }
            Combinator::AdjacentSibling => {
                let siblings = &parent_node.children;
                let node_idx = match siblings.iter().position(|id| *id == node.id) {
                    Some(idx) => idx,
                    None => return false,
                };
                if node_idx == 0 {
                    return false;
                }

                let previous_sibling_node = match tree.get_node(&siblings[node_idx - 1]) {
                    Some(node) => node,
                    None => return false,
                };

                let sibling_element = match previous_sibling_node.data.as_element() {
                    Some(elem) => elem,
                    None => return false,
                };
                let class_set = sibling_element.class_set.as_ref();
                matches_from_index(sequences, index - 1, tree, previous_sibling_node, class_set)
            }
            Combinator::GeneralSibling => {
                let siblings = &parent_node.children;
                let node_idx = match siblings.iter().position(|id| *id == node.id) {
                    Some(idx) => idx,
                    None => return false,
                };

                for sibling_id in &siblings[..node_idx] {
                    let sibling_node = match tree.get_node(sibling_id) {
                        Some(node) => node,
                        None => continue,
                    };

                    let Some(sibling_element) = sibling_node.data.as_element() else {
                        continue;
                    };
                    let class_set = sibling_element.class_set.as_ref();
                    if matches_from_index(sequences, index - 1, tree, sibling_node, class_set) {
                        return true;
                    }
                }

                false
            }
        }
    }

    matches_from_index(sequence, sequence.len() - 1, tree, node, class_set)
}
