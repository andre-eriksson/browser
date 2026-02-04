use css_cssom::{CssToken, CssTokenKind, HashType};
use html_dom::{DocumentRoot, DomNode, Element, HtmlTag, NodeId, Tag};

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
fn matches_compound_selectors(compound_selectors: &[CompoundSelector], element: &Element) -> bool {
    for compound_selector in compound_selectors {
        if !matches_simple_selectors(&compound_selector.tokens, element) {
            return false;
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

            let sensitivity = match &attribute_selector.case {
                Some(case) => case,
                None => &CaseSensitivity::default(),
            };

            match operator {
                AttributeOperator::Equals => match sensitivity {
                    CaseSensitivity::CaseInsensitive => {
                        if !element_attribute_value.eq_ignore_ascii_case(expected_value.as_str()) {
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
                        let words: Vec<&str> = element_attribute_value.split_whitespace().collect();
                        if !words
                            .iter()
                            .any(|word| word.eq_ignore_ascii_case(expected_value.as_str()))
                        {
                            return false;
                        }
                    }
                    CaseSensitivity::CaseSensitive => {
                        let words: Vec<&str> = element_attribute_value.split_whitespace().collect();
                        if !words.contains(&expected_value.as_str()) {
                            return false;
                        }
                    }
                },
                AttributeOperator::DashMatch => match sensitivity {
                    CaseSensitivity::CaseInsensitive => {
                        if !element_attribute_value.eq_ignore_ascii_case(expected_value.as_str())
                            && !element_attribute_value
                                .to_lowercase()
                                .starts_with(&format!("{}-", expected_value.to_lowercase()))
                        {
                            return false;
                        }
                    }
                    CaseSensitivity::CaseSensitive => {
                        if element_attribute_value != *expected_value
                            && !element_attribute_value.starts_with(&format!("{}-", expected_value))
                        {
                            return false;
                        }
                    }
                },
                AttributeOperator::PrefixMatch => match sensitivity {
                    CaseSensitivity::CaseInsensitive => {
                        if !element_attribute_value
                            .to_lowercase()
                            .starts_with(&expected_value.to_lowercase())
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
                        if !element_attribute_value
                            .to_lowercase()
                            .ends_with(&expected_value.to_lowercase())
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
                            .to_lowercase()
                            .contains(&expected_value.to_lowercase())
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
fn matches_simple_selectors(simple_selectors: &[CssToken], element: &Element) -> bool {
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
                        None | Some(CssTokenKind::Delim(_)) | Some(CssTokenKind::Whitespace) => {
                            if element.tag_name().to_lowercase() != ident.to_lowercase()
                                && ident != "*"
                            {
                                return false;
                            }
                        }
                        _ => {}
                    }
                } else if let Some(CssTokenKind::Delim(delim)) = prev {
                    if delim == &'.'
                        && !element
                            .classes()
                            .any(|class| class.eq_ignore_ascii_case(ident))
                    {
                        return false;
                    }
                } else if let Some(CssTokenKind::Colon) = prev {
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
                        return element.tag == Tag::Html(HtmlTag::A)
                            && element.has_attribute("href");
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
) -> bool {
    let element = match node.data.as_element() {
        Some(elem) => elem,
        None => return false,
    };

    for sequence in sequence.iter().rev() {
        let matched = matches_compound_selectors(&sequence.compound_selectors, element);

        if sequence.combinator.is_none() && !matched {
            return false;
        } else if sequence.combinator.is_none() && matched {
            continue;
        }

        let parent = match node.parent {
            Some(parent_id) => parent_id,
            None => return false,
        };

        let mut parent_node = match tree.get_node(&parent) {
            Some(node) => node,
            None => return false,
        };

        let parent_element = match parent_node.data.as_element() {
            Some(elem) => elem,
            None => return false,
        };

        match &sequence.combinator {
            Some(Combinator::Child) => {
                return matches_compound_selectors(&sequence.compound_selectors, parent_element);
            }
            Some(Combinator::Descendant) => {
                if matches_compound_selectors(&sequence.compound_selectors, parent_element) {
                    return true;
                }

                while let Some(grandparent_id) = parent_node.parent {
                    let grandparent_node = match tree.get_node(&grandparent_id) {
                        Some(node) => node,
                        None => break,
                    };

                    let grandparent_element = match grandparent_node.data.as_element() {
                        Some(elem) => elem,
                        None => break,
                    };

                    if matches_compound_selectors(&sequence.compound_selectors, grandparent_element)
                    {
                        return true;
                    }

                    parent_node = grandparent_node;
                }

                return false;
            }
            Some(Combinator::AdjacentSibling) => {
                let siblings = &parent_node.children;

                let mut found_current = false;
                let mut previous_sibling_id: Option<NodeId> = None;
                for &sibling_id in siblings {
                    if sibling_id == node.id {
                        found_current = true;
                    }

                    if !found_current {
                        previous_sibling_id = Some(sibling_id);
                        continue;
                    }

                    if previous_sibling_id.is_none() {
                        break;
                    }

                    let previous_sibling_node = match tree.get_node(&previous_sibling_id.unwrap()) {
                        Some(node) => node,
                        None => continue,
                    };

                    let previous_sibling_element = match previous_sibling_node.data.as_element() {
                        Some(elem) => elem,
                        None => continue,
                    };

                    return matches_compound_selectors(
                        &sequence.compound_selectors,
                        previous_sibling_element,
                    );
                }
            }
            Some(Combinator::GeneralSibling) => {
                let siblings = &parent_node.children;

                for &sibling_id in siblings {
                    if sibling_id == node.id {
                        break;
                    }

                    let sibling_node = match tree.get_node(&sibling_id) {
                        Some(node) => node,
                        None => continue,
                    };

                    let sibling_element = match sibling_node.data.as_element() {
                        Some(elem) => elem,
                        None => continue,
                    };

                    if matches_compound_selectors(&sequence.compound_selectors, sibling_element) {
                        return true;
                    }
                }
            }
            None => {}
        }
    }

    true
}
