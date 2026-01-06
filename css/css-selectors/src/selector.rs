use css_cssom::{CssToken, CssTokenKind};
use html_syntax::dom::{DocumentRoot, DomNode, Element, NodeId};

/// A sequence of simple selectors connected by combinators
#[derive(Debug)]
pub struct SelectorSequence {
    /// A list of simple selectors in this sequence
    simple_selectors: Vec<CssToken>,

    /// The combinator that connects this sequence to the next one
    combinator: Option<Combinator>,
}

/// The combinators used in CSS selectors
#[derive(Debug)]
pub enum Combinator {
    /// The descendant combinator (a space)
    Descendant,

    /// The child combinator ('>')
    Child,

    /// The adjacent sibling combinator ('+')
    AdjacentSibling,

    /// The general sibling combinator ('~')
    GeneralSibling,
}

/// Generate selector sequences from a list of CSS tokens
///
/// # Arguments
/// * `css_tokens` - A vector of CSS tokens representing the selector
///
/// # Returns
/// * `Vec<SelectorSequence>` - A vector of selector sequences
pub fn generate_sequences(css_tokens: Vec<CssToken>) -> Vec<SelectorSequence> {
    let mut sequences: Vec<SelectorSequence> = Vec::new();
    let mut current_sequence = SelectorSequence {
        simple_selectors: Vec::new(),
        combinator: None,
    };

    for token in css_tokens.iter() {
        match token.kind {
            CssTokenKind::Delim('>') => {
                current_sequence.combinator = Some(Combinator::Child);
                sequences.push(current_sequence);
                current_sequence = SelectorSequence {
                    simple_selectors: Vec::new(),
                    combinator: None,
                };
            }
            CssTokenKind::Delim('+') => {
                current_sequence.combinator = Some(Combinator::AdjacentSibling);
                sequences.push(current_sequence);
                current_sequence = SelectorSequence {
                    simple_selectors: Vec::new(),
                    combinator: None,
                };
            }
            CssTokenKind::Delim('~') => {
                current_sequence.combinator = Some(Combinator::GeneralSibling);
                sequences.push(current_sequence);
                current_sequence = SelectorSequence {
                    simple_selectors: Vec::new(),
                    combinator: None,
                };
            }
            CssTokenKind::Ident(_) => {
                if let Some(previous_token) = current_sequence.simple_selectors.last()
                    && previous_token.kind == CssTokenKind::Whitespace
                    && current_sequence.simple_selectors.len() > 1
                {
                    current_sequence.simple_selectors.pop();
                    current_sequence.combinator = Some(Combinator::Descendant);
                    sequences.push(current_sequence);
                    current_sequence = SelectorSequence {
                        simple_selectors: Vec::new(),
                        combinator: None,
                    };
                }

                current_sequence.simple_selectors.push(token.clone());
            }
            _ => {
                current_sequence.simple_selectors.push(token.clone());
            }
        }
    }

    if !current_sequence.simple_selectors.is_empty() {
        sequences.push(current_sequence);
    }

    sequences
}

/// Check if an element matches a list of simple selectors
///
/// # Arguments
/// * `simple_selectors` - A slice of CSS tokens representing simple selectors
/// * `element` - The DOM element to check for a match
///
/// # Returns
/// * `bool` - True if the element matches the simple selectors, false otherwise
pub fn matches_simple_selectors(simple_selectors: &[CssToken], element: &Element) -> bool {
    for i in 0..simple_selectors.len() {
        let previous_token = &simple_selectors.get(i.wrapping_sub(1));
        let current_token = &simple_selectors[i];
        let next_token = simple_selectors.get(i + 1);

        if let CssTokenKind::Ident(ident) = &current_token.kind {
            let prev = previous_token.map(|t| &t.kind);
            let next = next_token.map(|t| &t.kind);

            if prev.is_none() || matches!(prev, Some(CssTokenKind::Whitespace)) {
                match next {
                    None | Some(CssTokenKind::Delim(_)) | Some(CssTokenKind::Whitespace) => {
                        if element.tag_name().to_string().to_lowercase() != ident.to_lowercase() {
                            return false;
                        }
                    }
                    _ => {}
                }
            } else if let Some(CssTokenKind::Delim(delim)) = prev {
                match delim {
                    '.' => {
                        if !element
                            .classes()
                            .any(|class| class.eq_ignore_ascii_case(ident))
                        {
                            return false;
                        }
                    }
                    '#' => {
                        if !element.id().is_none_or(|id| id.eq_ignore_ascii_case(ident)) {
                            return false;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    true
}

/// Check if a node matches a list of selector sequences
///
/// # Arguments
/// * `sequences` - A vector of selector sequences to match against
/// * `tree` - The document root containing the DOM tree
/// * `node` - The DOM node to check for a match
///
/// # Returns
/// * `bool` - True if the node matches the selector sequences, false otherwise
pub fn matches(sequences: Vec<SelectorSequence>, tree: &DocumentRoot, node: &DomNode) -> bool {
    let element = match node.data.as_element() {
        Some(elem) => elem,
        None => return false,
    };

    for sequence in sequences.iter().rev() {
        let matched = matches_simple_selectors(&sequence.simple_selectors, element);
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

        // Handle combinators
        match &sequence.combinator {
            Some(Combinator::Child) => {
                // Move to parent node
                return matches_simple_selectors(&sequence.simple_selectors, parent_element);
            }
            Some(Combinator::Descendant) => {
                // Move to ancestor nodes
                if matches_simple_selectors(&sequence.simple_selectors, parent_element) {
                    return true; // ?
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

                    if matches_simple_selectors(&sequence.simple_selectors, grandparent_element) {
                        return true;
                    }

                    // Move up the tree
                    parent_node = grandparent_node;
                }

                return false;
            }
            Some(Combinator::AdjacentSibling) => {
                // Move to next sibling node
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

                    return matches_simple_selectors(
                        &sequence.simple_selectors,
                        previous_sibling_element,
                    );
                }
            }
            Some(Combinator::GeneralSibling) => {
                // Move to any previous sibling nodes
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

                    if matches_simple_selectors(&sequence.simple_selectors, sibling_element) {
                        return true;
                    }
                }
            }
            None => {}
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use html_syntax::{
        dom::{Element, NodeData, NodeId},
        tag::{HtmlTag, KnownTag},
    };

    #[test]
    fn generate_sequences_test() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Ident("div".to_string()),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Delim('>'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("p".to_string()),
                position: None,
            },
        ];

        let sequences = generate_sequences(selector_tokens);

        assert_eq!(sequences.len(), 2);
        assert_eq!(sequences[0].simple_selectors.len(), 1);
        assert_eq!(sequences[1].simple_selectors.len(), 1);
        assert!(matches!(sequences[0].combinator, Some(Combinator::Child)));
        assert!(sequences[1].combinator.is_none());
    }

    #[test]
    fn test_matches_type_selector() {
        let selector_tokens = vec![CssToken {
            kind: CssTokenKind::Ident("div".to_string()),
            position: None,
        }];

        let tree = DocumentRoot::new();

        let node_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), HashMap::new()));

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        let selectors = generate_sequences(selector_tokens);

        assert!(matches(selectors, &tree, &node));
    }

    #[test]
    fn test_matches_fail_type_selector() {
        let selector_tokens = vec![CssToken {
            kind: CssTokenKind::Ident("div".to_string()),
            position: None,
        }];

        let tree = DocumentRoot::new();

        let node_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::P), HashMap::new()));

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        let selectors = generate_sequences(selector_tokens);

        assert!(!matches(selectors, &tree, &node));
    }

    #[test]
    fn test_matches_id_selector() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Delim('#'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("test-id".to_string()),
                position: None,
            },
        ];

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("id".to_string(), "test-id".to_string());

        let node_data = NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), attributes));

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        let selectors = generate_sequences(selector_tokens);

        assert!(matches(selectors, &tree, &node));
    }

    #[test]
    fn test_matches_fail_id_selector() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Delim('#'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("test-id".to_string()),
                position: None,
            },
        ];

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("id".to_string(), "wrong-id".to_string());

        let node_data = NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), attributes));

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        let selectors = generate_sequences(selector_tokens);

        assert!(!matches(selectors, &tree, &node));
    }

    #[test]
    fn test_matches_class_selector() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Delim('.'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("test-class".to_string()),
                position: None,
            },
        ];

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("class".to_string(), "test-class another-class".to_string());

        let node_data = NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), attributes));

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        let selectors = generate_sequences(selector_tokens);

        assert!(matches(selectors, &tree, &node));
    }

    #[test]
    fn test_matches_fail_class_selector() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Delim('.'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("test-class".to_string()),
                position: None,
            },
        ];

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("class".to_string(), "wrong-class another-class".to_string());

        let node_data = NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), attributes));

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        let selectors = generate_sequences(selector_tokens);

        assert!(!matches(selectors, &tree, &node));
    }

    #[test]
    fn test_matches_type_and_id_selector() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Ident("div".to_string()),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Delim('#'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("test-id".to_string()),
                position: None,
            },
        ];

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("id".to_string(), "test-id".to_string());

        let node_data = NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), attributes));

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        let selectors = generate_sequences(selector_tokens);

        assert!(matches(selectors, &tree, &node));
    }

    #[test]
    fn test_matches_fail_type_and_id_selector() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Ident("div".to_string()),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Delim('#'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("test-id".to_string()),
                position: None,
            },
        ];

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("id".to_string(), "wrong-id".to_string());

        let node_data = NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), attributes));

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        let selectors = generate_sequences(selector_tokens);
        assert!(!matches(selectors, &tree, &node));
    }

    #[test]
    fn test_matches_type_and_class_selector() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Ident("div".to_string()),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Delim('.'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("test-class".to_string()),
                position: None,
            },
        ];

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("class".to_string(), "test-class another-class".to_string());

        let node_data = NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), attributes));

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        let selectors = generate_sequences(selector_tokens);

        assert!(matches(selectors, &tree, &node));
    }

    #[test]
    fn test_matches_fail_type_and_class_selector() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Ident("div".to_string()),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Delim('.'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("test-class".to_string()),
                position: None,
            },
        ];

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("class".to_string(), "wrong-class another-class".to_string());

        let node_data = NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), attributes));
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        let selectors = generate_sequences(selector_tokens);
        assert!(!matches(selectors, &tree, &node));
    }

    #[test]
    fn test_matches_multiple_class_selectors() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Delim('.'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("class-one".to_string()),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Delim('.'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("class-two".to_string()),
                position: None,
            },
        ];

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert(
            "class".to_string(),
            "class-one class-two class-three".to_string(),
        );

        let node_data = NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), attributes));

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        let selectors = generate_sequences(selector_tokens);

        assert!(matches(selectors, &tree, &node));
    }

    #[test]
    fn test_matches_fail_multiple_class_selectors() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Delim('.'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("class-one".to_string()),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Delim('.'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("class-two".to_string()),
                position: None,
            },
        ];

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("class".to_string(), "class-one class-three".to_string());
        let node_data = NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), attributes));

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        let selectors = generate_sequences(selector_tokens);
        assert!(!matches(selectors, &tree, &node));
    }

    #[test]
    fn test_matches_child_combinator() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Ident("div".to_string()),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Delim('>'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("p".to_string()),
                position: None,
            },
        ];

        let mut tree = DocumentRoot::new();

        let parent_node_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), HashMap::new()));
        let parent_node_id = tree.push_node(parent_node_data, None);
        let child_node_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::P), HashMap::new()));
        let child_node_id = tree.push_node(child_node_data, Some(parent_node_id));
        let child_node = tree.get_node(&child_node_id).unwrap();

        let selectors = generate_sequences(selector_tokens);

        assert!(matches(selectors, &tree, child_node));
    }

    #[test]
    fn test_matches_fail_child_combinator() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Ident("div".to_string()),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Delim('>'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("p".to_string()),
                position: None,
            },
        ];

        let mut tree = DocumentRoot::new();

        let parent_node_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), HashMap::new()));
        let parent_node_id = tree.push_node(parent_node_data, None);
        let child_node_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::A), HashMap::new()));
        let child_node_id = tree.push_node(child_node_data, Some(parent_node_id));
        let child_node = tree.get_node(&child_node_id).unwrap();

        let selectors = generate_sequences(selector_tokens);

        assert!(!matches(selectors, &tree, child_node));
    }

    #[test]
    fn test_matches_descendant_combinator() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Ident("div".to_string()),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("p".to_string()),
                position: None,
            },
        ];

        let mut tree = DocumentRoot::new();

        let grandparent_node_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), HashMap::new()));
        let grandparent_node_id = tree.push_node(grandparent_node_data, None);
        let parent_node_data = NodeData::Element(Element::new(
            HtmlTag::Known(KnownTag::Section),
            HashMap::new(),
        ));
        let parent_node_id = tree.push_node(parent_node_data, Some(grandparent_node_id));
        let child_node_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::P), HashMap::new()));
        let child_node_id = tree.push_node(child_node_data, Some(parent_node_id));
        let child_node = tree.get_node(&child_node_id).unwrap();

        let selectors = generate_sequences(selector_tokens);

        assert!(matches(selectors, &tree, child_node));
    }

    #[test]
    fn test_matches_fail_descendant_combinator() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Ident("div".to_string()),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("p".to_string()),
                position: None,
            },
        ];

        let mut tree = DocumentRoot::new();

        let grandparent_node_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), HashMap::new()));
        let grandparent_node_id = tree.push_node(grandparent_node_data, None);
        let parent_node_data = NodeData::Element(Element::new(
            HtmlTag::Known(KnownTag::Section),
            HashMap::new(),
        ));
        let parent_node_id = tree.push_node(parent_node_data, Some(grandparent_node_id));
        let child_node_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::A), HashMap::new()));
        let child_node_id = tree.push_node(child_node_data, Some(parent_node_id));
        let child_node = tree.get_node(&child_node_id).unwrap();

        let selectors = generate_sequences(selector_tokens);

        assert!(!matches(selectors, &tree, child_node));
    }

    #[test]
    fn test_matches_adjacent_sibling_combinator() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Ident("div".to_string()),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Delim('+'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("p".to_string()),
                position: None,
            },
        ];

        let mut tree = DocumentRoot::new();
        let parent_node_data = NodeData::Element(Element::new(
            HtmlTag::Known(KnownTag::Section),
            HashMap::new(),
        ));
        let parent_node_id = tree.push_node(parent_node_data, None);
        let first_sibling_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), HashMap::new()));
        tree.push_node(first_sibling_data, Some(parent_node_id));
        let second_sibling_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::P), HashMap::new()));
        let second_sibling_id = tree.push_node(second_sibling_data, Some(parent_node_id));
        let second_sibling_node = tree.get_node(&second_sibling_id).unwrap();

        let selectors = generate_sequences(selector_tokens);

        assert!(matches(selectors, &tree, second_sibling_node));
    }

    #[test]
    fn test_matches_fail_adjacent_sibling_combinator() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Ident("div".to_string()),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Delim('+'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("p".to_string()),
                position: None,
            },
        ];

        let mut tree = DocumentRoot::new();
        let parent_node_data = NodeData::Element(Element::new(
            HtmlTag::Known(KnownTag::Section),
            HashMap::new(),
        ));
        let parent_node_id = tree.push_node(parent_node_data, None);
        let first_sibling_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), HashMap::new()));
        tree.push_node(first_sibling_data, Some(parent_node_id));
        let second_sibling_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::A), HashMap::new()));
        let second_sibling_id = tree.push_node(second_sibling_data, Some(parent_node_id));
        let second_sibling_node = tree.get_node(&second_sibling_id).unwrap();

        let selectors = generate_sequences(selector_tokens);

        assert!(!matches(selectors, &tree, second_sibling_node));
    }

    #[test]
    fn test_matches_fail_multiple_adjacent_sibling_combinator() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Ident("div".to_string()),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Delim('+'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("p".to_string()),
                position: None,
            },
        ];

        let mut tree = DocumentRoot::new();
        let parent_node_data = NodeData::Element(Element::new(
            HtmlTag::Known(KnownTag::Section),
            HashMap::new(),
        ));
        let parent_node_id = tree.push_node(parent_node_data, None);

        let first_sibling_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), HashMap::new()));
        tree.push_node(first_sibling_data, Some(parent_node_id)); // 1

        let second_sibling_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Span), HashMap::new()));
        tree.push_node(second_sibling_data, Some(parent_node_id)); // 2

        let third_sibling_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::P), HashMap::new()));
        let third_sibling_id = tree.push_node(third_sibling_data, Some(parent_node_id)); // 3

        let third_sibling_node = tree.get_node(&third_sibling_id).unwrap();

        let selectors = generate_sequences(selector_tokens);

        assert!(!matches(selectors, &tree, third_sibling_node));
    }

    #[test]
    fn test_matches_general_sibling_combinator() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Ident("div".to_string()),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Delim('~'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("p".to_string()),
                position: None,
            },
        ];

        let mut tree = DocumentRoot::new();
        let parent_node_data = NodeData::Element(Element::new(
            HtmlTag::Known(KnownTag::Section),
            HashMap::new(),
        ));
        let parent_node_id = tree.push_node(parent_node_data, None);
        let first_sibling_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), HashMap::new()));
        tree.push_node(first_sibling_data, Some(parent_node_id));
        let second_sibling_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Span), HashMap::new()));
        tree.push_node(second_sibling_data, Some(parent_node_id));
        let third_sibling_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::P), HashMap::new()));
        let third_sibling_id = tree.push_node(third_sibling_data, Some(parent_node_id));
        let third_sibling_node = tree.get_node(&third_sibling_id).unwrap();

        let selectors = generate_sequences(selector_tokens);
        assert!(matches(selectors, &tree, third_sibling_node));
    }

    #[test]
    fn test_matches_fail_general_sibling_combinator() {
        let selector_tokens = vec![
            CssToken {
                kind: CssTokenKind::Ident("div".to_string()),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Delim('~'),
                position: None,
            },
            CssToken {
                kind: CssTokenKind::Ident("p".to_string()),
                position: None,
            },
        ];

        let mut tree = DocumentRoot::new();
        let parent_node_data = NodeData::Element(Element::new(
            HtmlTag::Known(KnownTag::Section),
            HashMap::new(),
        ));
        let parent_node_id = tree.push_node(parent_node_data, None);
        let first_sibling_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Div), HashMap::new()));
        tree.push_node(first_sibling_data, Some(parent_node_id));
        let second_sibling_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::Span), HashMap::new()));
        tree.push_node(second_sibling_data, Some(parent_node_id));
        let third_sibling_data =
            NodeData::Element(Element::new(HtmlTag::Known(KnownTag::A), HashMap::new()));
        let third_sibling_id = tree.push_node(third_sibling_data, Some(parent_node_id));
        let third_sibling_node = tree.get_node(&third_sibling_id).unwrap();

        let selectors = generate_sequences(selector_tokens);
        assert!(!matches(selectors, &tree, third_sibling_node));
    }
}
