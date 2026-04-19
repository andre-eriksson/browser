//! CSS Selectors Module
//!
//! This module provides functionality for parsing, matching, and calculating specificity of CSS selectors.

/// A module for matching CSS selectors against DOM nodes
mod matching;

/// A module for parsing CSS selectors
mod parser;

/// A module for CSS selector structures and generation
mod selector;

/// A module for calculating the specificity of CSS selectors
mod specificity;

pub use matching::{AttributeOperator, Combinator, matches_compound};
pub use parser::CaseSensitivity;
pub use selector::{AttributeSelector, CompoundSelector, CompoundSelectorSequence, generate_selector_list};
pub use specificity::{SelectorSpecificity, SpecificityCalculable};

#[cfg(test)]
#[allow(clippy::vec_init_then_push)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use css_cssom::{AssociatedToken, ComponentValue, CssToken, CssTokenKind, HashType, SimpleBlock};

    use crate::{SelectorSpecificity, SpecificityCalculable};
    use crate::{matching::matches_compound, selector::generate_compound_sequences};
    use html_dom::{DocumentRoot, DomNode, Element, HtmlTag, NodeData, NodeId, Tag};

    macro_rules! generate_compound_token {
        ($($kind:expr),* ; attr[ $($attr_kind:expr),* $(,)? ] $(; $($rest_kind:expr),*)? $(,)?) => {{
            let mut tokens = Vec::new();

            $(
                tokens.push(ComponentValue::Token(CssToken {
                    kind: $kind,
                    position: None,
                }));
            )*

            let mut attr_tokens = Vec::new();
            $(
                attr_tokens.push(ComponentValue::Token(CssToken {
                    kind: $attr_kind,
                    position: None,
                }));
            )*

            tokens.push(ComponentValue::SimpleBlock(SimpleBlock {
                associated_token: AssociatedToken::SquareBracket,
                value: attr_tokens,
            }));

            $($(
                tokens.push(ComponentValue::Token(CssToken {
                    kind: $rest_kind,
                    position: None,
                }));
            )*)?

            tokens
        }};

        ($($kind:expr),* $(,)?) => {{
            let mut tokens = Vec::new();
            $(
                tokens.push(ComponentValue::Token(CssToken {
                    kind: $kind,
                    position: None,
                }));
            )*
            tokens
        }};
    }

    macro_rules! generate_node_data {
        ($tag:expr, $hash_set:expr, $attributes:expr) => {{ NodeData::Element(Element::new(Tag::Html($tag), $hash_set, $attributes)) }};
    }

    #[test]
    fn generate_compound_sequences_test() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("a".to_string());
            attr[
                CssTokenKind::Ident("href".to_string()),
                CssTokenKind::Delim('='),
                CssTokenKind::String("example.com".to_string())
            ]
        );

        let sequences = generate_compound_sequences(&components);

        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 1));
        assert_eq!(sequences[0].compound_selectors.len(), 1);
        assert_eq!(sequences[0].compound_selectors[0].tokens.len(), 1);
        assert_eq!(sequences[0].compound_selectors[0].attribute_selectors.len(), 1);
    }

    // === Basic Selector Tests ===

    #[test]
    fn match_type_selector() {
        let components = generate_compound_token!(CssTokenKind::Ident("div".to_string()));

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 0, 1));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), HashMap::default());

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn no_match_type_selector() {
        let components = generate_compound_token!(CssTokenKind::Ident("span".to_string()));

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 0, 1));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), HashMap::default());

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn match_class_selector() {
        let components =
            generate_compound_token!(CssTokenKind::Delim('.'), CssTokenKind::Ident("my-class".to_string()));

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let tree = DocumentRoot::new();
        let mut hash_set = HashSet::new();
        hash_set.insert("my-class".to_string());
        hash_set.insert("another-class".to_string());

        let mut attributes = HashMap::new();
        attributes.insert("class".to_string(), "my-class another-class".to_string());

        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn no_match_class_selector() {
        let components =
            generate_compound_token!(CssTokenKind::Delim('.'), CssTokenKind::Ident("my-class".to_string()));

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let tree = DocumentRoot::new();

        let mut hash_set = HashSet::new();
        hash_set.insert("wrong-class".to_string());
        hash_set.insert("another-class".to_string());

        let mut attributes = HashMap::new();
        attributes.insert("class".to_string(), "wrong-class another-class".to_string());

        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn match_id_selector() {
        let components = generate_compound_token!(CssTokenKind::Hash {
            value: "my-id".to_string(),
            type_flag: HashType::Id,
        });

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(1, 0, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let mut attributes = HashMap::new();
        attributes.insert("id".to_string(), "my-id".to_string());

        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn no_match_id_selector() {
        let components = generate_compound_token!(CssTokenKind::Hash {
            value: "my-id".to_string(),
            type_flag: HashType::Id,
        });

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(1, 0, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let mut attributes = HashMap::new();
        attributes.insert("id".to_string(), "wrong-id".to_string());

        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn match_type_and_class_selector() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("div".to_string()),
            CssTokenKind::Delim('.'),
            CssTokenKind::Ident("my-class".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 1));

        let tree = DocumentRoot::new();
        let mut hash_set = HashSet::new();
        hash_set.insert("my-class".to_string());
        hash_set.insert("another-class".to_string());

        let mut attributes = HashMap::new();
        attributes.insert("class".to_string(), "my-class another-class".to_string());

        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn no_match_type_and_class_selector() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("div".to_string()),
            CssTokenKind::Delim('.'),
            CssTokenKind::Ident("my-class".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 1));

        let tree = DocumentRoot::new();

        let mut hash_set = HashSet::new();
        hash_set.insert("wrong-class".to_string());
        hash_set.insert("another-class".to_string());

        let mut attributes = HashMap::new();
        attributes.insert("class".to_string(), "wrong-class another-class".to_string());

        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    // === Ancestor Combinator Tests ===

    #[test]
    fn match_descendant_combinator() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("div".to_string()),
            CssTokenKind::Whitespace,
            CssTokenKind::Ident("span".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 2);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 0, 1));
        assert_eq!(sequences[1].specificity(), SelectorSpecificity::new(0, 0, 1));

        let mut tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let grandparent_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), HashMap::default());
        let grandparent_id = tree.push_node(&grandparent_data, None);

        let parent_data = generate_node_data!(HtmlTag::Section, hash_set.clone(), HashMap::default());
        let parent_id = tree.push_node(&parent_data, Some(grandparent_id));

        let child_data = generate_node_data!(HtmlTag::Span, hash_set.clone(), HashMap::default());
        let child_id = tree.push_node(&child_data, Some(parent_id));

        let child_node = tree.get_node(&child_id).unwrap();

        assert!(matches_compound(&sequences, &tree, child_node, Some(&hash_set)));
    }

    #[test]
    fn no_match_descendant_combinator() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("div".to_string()),
            CssTokenKind::Whitespace,
            CssTokenKind::Ident("span".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 2);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 0, 1));
        assert_eq!(sequences[1].specificity(), SelectorSpecificity::new(0, 0, 1));

        let mut tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let grandparent_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), HashMap::default());
        let grandparent_id = tree.push_node(&grandparent_data, None);

        let parent_data = generate_node_data!(HtmlTag::Section, hash_set.clone(), HashMap::default());
        let parent_id = tree.push_node(&parent_data, Some(grandparent_id));

        let child_data = generate_node_data!(HtmlTag::P, hash_set.clone(), HashMap::default());
        let child_id = tree.push_node(&child_data, Some(parent_id));

        let child_node = tree.get_node(&child_id).unwrap();

        assert!(!matches_compound(&sequences, &tree, child_node, Some(&hash_set)));
    }

    #[test]
    fn match_child_combinator() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("div".to_string()),
            CssTokenKind::Delim('>'),
            CssTokenKind::Ident("span".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 2);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 0, 1));
        assert_eq!(sequences[1].specificity(), SelectorSpecificity::new(0, 0, 1));

        let mut tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let parent_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), HashMap::default());
        let parent_id = tree.push_node(&parent_data, None);

        let child_data = generate_node_data!(HtmlTag::Span, hash_set.clone(), HashMap::default());
        let child_id = tree.push_node(&child_data, Some(parent_id));

        let child_node = tree.get_node(&child_id).unwrap();

        assert!(matches_compound(&sequences, &tree, child_node, Some(&hash_set)));
    }

    #[test]
    fn no_match_child_combinator() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("div".to_string()),
            CssTokenKind::Delim('>'),
            CssTokenKind::Ident("span".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 2);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 0, 1));
        assert_eq!(sequences[1].specificity(), SelectorSpecificity::new(0, 0, 1));

        let mut tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let parent_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), HashMap::default());
        let parent_id = tree.push_node(&parent_data, None);

        let child_data = generate_node_data!(HtmlTag::P, hash_set.clone(), HashMap::default());
        let child_id = tree.push_node(&child_data, Some(parent_id));

        let child_node = tree.get_node(&child_id).unwrap();

        assert!(!matches_compound(&sequences, &tree, child_node, Some(&hash_set)));
    }

    #[test]
    fn match_adjacent_sibling_combinator() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("div".to_string()),
            CssTokenKind::Delim('+'),
            CssTokenKind::Ident("span".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 2);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 0, 1));
        assert_eq!(sequences[1].specificity(), SelectorSpecificity::new(0, 0, 1));

        let mut tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let parent_data = generate_node_data!(HtmlTag::Section, hash_set.clone(), HashMap::default());
        let parent_id = tree.push_node(&parent_data, None);

        let sibling1_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), HashMap::default());
        tree.push_node(&sibling1_data, Some(parent_id));

        let sibling2_data = generate_node_data!(HtmlTag::Span, hash_set.clone(), HashMap::default());
        let sibling2_id = tree.push_node(&sibling2_data, Some(parent_id));

        let sibling2_node = tree.get_node(&sibling2_id).unwrap();

        assert!(matches_compound(&sequences, &tree, sibling2_node, Some(&hash_set)));
    }

    #[test]
    fn no_match_adjacent_sibling_combinator() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("div".to_string()),
            CssTokenKind::Delim('+'),
            CssTokenKind::Ident("span".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 2);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 0, 1));
        assert_eq!(sequences[1].specificity(), SelectorSpecificity::new(0, 0, 1));

        let mut tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let parent_data = generate_node_data!(HtmlTag::Section, hash_set.clone(), HashMap::default());
        let parent_id = tree.push_node(&parent_data, None);

        let sibling1_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), HashMap::default());
        tree.push_node(&sibling1_data, Some(parent_id));

        let sibling2_data = generate_node_data!(HtmlTag::P, hash_set.clone(), HashMap::default());
        let sibling2_id = tree.push_node(&sibling2_data, Some(parent_id));

        let sibling2_node = tree.get_node(&sibling2_id).unwrap();

        assert!(!matches_compound(&sequences, &tree, sibling2_node, Some(&hash_set)));
    }

    #[test]
    fn too_distant_no_match_adjacent_sibling_combinator() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("div".to_string()),
            CssTokenKind::Delim('+'),
            CssTokenKind::Ident("span".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 2);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 0, 1));
        assert_eq!(sequences[1].specificity(), SelectorSpecificity::new(0, 0, 1));

        let mut tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let parent_data = generate_node_data!(HtmlTag::Section, hash_set.clone(), HashMap::default());
        let parent_id = tree.push_node(&parent_data, None);

        let sibling1_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), HashMap::default());
        tree.push_node(&sibling1_data, Some(parent_id));

        let sibling2_data = generate_node_data!(HtmlTag::P, hash_set.clone(), HashMap::default());
        tree.push_node(&sibling2_data, Some(parent_id));

        let sibling3_data = generate_node_data!(HtmlTag::Span, hash_set.clone(), HashMap::default());
        let sibling3_id = tree.push_node(&sibling3_data, Some(parent_id));

        let sibling3_node = tree.get_node(&sibling3_id).unwrap();

        assert!(!matches_compound(&sequences, &tree, sibling3_node, Some(&hash_set)));
    }

    #[test]
    fn match_sibling_combinator() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("div".to_string()),
            CssTokenKind::Delim('~'),
            CssTokenKind::Ident("span".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 2);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 0, 1));
        assert_eq!(sequences[1].specificity(), SelectorSpecificity::new(0, 0, 1));

        let mut tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let parent_data = generate_node_data!(HtmlTag::Section, hash_set.clone(), HashMap::default());
        let parent_id = tree.push_node(&parent_data, None);

        let sibling1_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), HashMap::default());
        tree.push_node(&sibling1_data, Some(parent_id));

        let sibling2_data = generate_node_data!(HtmlTag::Span, hash_set.clone(), HashMap::default());
        let sibling2_id = tree.push_node(&sibling2_data, Some(parent_id));

        let sibling2_node = tree.get_node(&sibling2_id).unwrap();

        assert!(matches_compound(&sequences, &tree, sibling2_node, Some(&hash_set)));
    }

    #[test]
    fn no_match_sibling_combinator() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("div".to_string()),
            CssTokenKind::Delim('~'),
            CssTokenKind::Ident("span".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 2);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 0, 1));
        assert_eq!(sequences[1].specificity(), SelectorSpecificity::new(0, 0, 1));

        let mut tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let parent_data = generate_node_data!(HtmlTag::Section, hash_set.clone(), HashMap::default());
        let parent_id = tree.push_node(&parent_data, None);

        let sibling1_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), HashMap::default());
        tree.push_node(&sibling1_data, Some(parent_id));

        let sibling2_data = generate_node_data!(HtmlTag::P, hash_set.clone(), HashMap::default());
        let sibling2_id = tree.push_node(&sibling2_data, Some(parent_id));

        let sibling2_node = tree.get_node(&sibling2_id).unwrap();

        assert!(!matches_compound(&sequences, &tree, sibling2_node, Some(&hash_set)));
    }

    #[test]
    fn distant_match_sibling_combinator() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("div".to_string()),
            CssTokenKind::Delim('~'),
            CssTokenKind::Ident("span".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 2);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 0, 1));
        assert_eq!(sequences[1].specificity(), SelectorSpecificity::new(0, 0, 1));

        let mut tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let parent_data = generate_node_data!(HtmlTag::Section, hash_set.clone(), HashMap::default());
        let parent_id = tree.push_node(&parent_data, None);

        let sibling1_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), HashMap::default());
        tree.push_node(&sibling1_data, Some(parent_id));

        let sibling2_data = generate_node_data!(HtmlTag::P, hash_set.clone(), HashMap::default());
        tree.push_node(&sibling2_data, Some(parent_id));

        let sibling3_data = generate_node_data!(HtmlTag::Span, hash_set.clone(), HashMap::default());
        let sibling3_id = tree.push_node(&sibling3_data, Some(parent_id));

        let sibling3_node = tree.get_node(&sibling3_id).unwrap();

        assert!(matches_compound(&sequences, &tree, sibling3_node, Some(&hash_set)));
    }

    // === Attribute Selector Tests ===

    #[test]
    fn match_attribute_selector_no_value() {
        let components = generate_compound_token!(
            ;
            attr[
                CssTokenKind::Ident("data-active".to_string()),
            ]
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-active".to_string(), "true".to_string());

        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn no_match_attribute_selector_no_value() {
        let components = generate_compound_token!(
            ;
            attr[
                CssTokenKind::Ident("data-active".to_string()),
            ]
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-inactive".to_string(), "true".to_string());

        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn match_attribute_selector_equals() {
        let components = generate_compound_token!(
            ;
            attr[
                CssTokenKind::Ident("data-test".to_string()),
                CssTokenKind::Delim('='),
                CssTokenKind::String("value".to_string()),
            ]
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-test".to_string(), "value".to_string());

        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn no_match_attribute_selector_equals() {
        let components = generate_compound_token!(
            ;
            attr[
                CssTokenKind::Ident("data-test".to_string()),
                CssTokenKind::Delim('='),
                CssTokenKind::String("value".to_string()),
            ]
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let mut attributes = HashMap::new();
        attributes.insert("data".to_string(), "value".to_string());

        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn match_attribute_selector_includes() {
        let components = generate_compound_token!(
            ;
            attr[
                CssTokenKind::Ident("data-tags".to_string()),
                CssTokenKind::Delim('~'),
                CssTokenKind::Delim('='),
                CssTokenKind::Ident("featured".to_string()),
            ]
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-tags".to_string(), "new featured popular".to_string());

        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn no_match_attribute_selector_includes() {
        let components = generate_compound_token!(
            ;
            attr[
                CssTokenKind::Ident("data-tags".to_string()),
                CssTokenKind::Delim('~'),
                CssTokenKind::Delim('='),
                CssTokenKind::Ident("featured".to_string()),
            ]
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-tags".to_string(), "new discount popular".to_string());

        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn match_attribute_selector_dash_match() {
        let components = generate_compound_token!(
            ;
            attr[
                CssTokenKind::Ident("data-region".to_string()),
                CssTokenKind::Delim('|'),
                CssTokenKind::Delim('='),
                CssTokenKind::Ident("us".to_string()),
            ]
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-region".to_string(), "us-west".to_string());
        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn no_match_attribute_selector_dash_match() {
        let components = generate_compound_token!(
            ;
            attr[
                CssTokenKind::Ident("data-region".to_string()),
                CssTokenKind::Delim('|'),
                CssTokenKind::Delim('='),
                CssTokenKind::Ident("us".to_string()),
            ]
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-region".to_string(), "eu-north1".to_string());
        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn match_attribute_selector_prefix_match() {
        let components = generate_compound_token!(
            ;
            attr[
                CssTokenKind::Ident("data-lang".to_string()),
                CssTokenKind::Delim('^'),
                CssTokenKind::Delim('='),
                CssTokenKind::Ident("en".to_string()),
            ]
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-lang".to_string(), "en-US".to_string());
        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn no_match_attribute_selector_prefix_match() {
        let components = generate_compound_token!(
            ;
            attr[
                CssTokenKind::Ident("data-lang".to_string()),
                CssTokenKind::Delim('^'),
                CssTokenKind::Delim('='),
                CssTokenKind::Ident("en".to_string()),
            ]
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-lang".to_string(), "sv-SE".to_string());
        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn match_attribute_selector_suffix_match() {
        let components = generate_compound_token!(
            ;
            attr[
                CssTokenKind::Ident("data-file".to_string()),
                CssTokenKind::Delim('$'),
                CssTokenKind::Delim('='),
                CssTokenKind::Ident(".jpg".to_string()),
            ]
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-file".to_string(), "image.jpg".to_string());
        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn no_match_attribute_selector_suffix_match() {
        let components = generate_compound_token!(
            ;
            attr[
                CssTokenKind::Ident("data-file".to_string()),
                CssTokenKind::Delim('$'),
                CssTokenKind::Delim('='),
                CssTokenKind::Ident(".jpg".to_string()),
            ]
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-file".to_string(), "document.pdf".to_string());
        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn match_attribute_selector_substring_match() {
        let components = generate_compound_token!(
            ;
            attr[
                CssTokenKind::Ident("data-info".to_string()),
                CssTokenKind::Delim('*'),
                CssTokenKind::Delim('='),
                CssTokenKind::Ident("middle".to_string()),
            ]
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-info".to_string(), "start-middle-end".to_string());
        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn no_match_attribute_selector_substring_match() {
        let components = generate_compound_token!(
            ;
            attr[
                CssTokenKind::Ident("data-info".to_string()),
                CssTokenKind::Delim('*'),
                CssTokenKind::Delim('='),
                CssTokenKind::Ident("middle".to_string()),
            ]
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let hash_set = HashSet::new();

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-info".to_string(), "start-end".to_string());
        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn match_attribute_selector_case_insensitive() {
        let components = generate_compound_token!(
            ;
            attr[
                CssTokenKind::Ident("data-lang".to_string()),
                CssTokenKind::Delim('='),
                CssTokenKind::String("EN".to_string()),
                CssTokenKind::Ident("i".to_string()),
            ]
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-lang".to_string(), "en".to_string());
        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn match_attribute_selector_case_sensitive() {
        let components = generate_compound_token!(
            ;
            attr[
                CssTokenKind::Ident("data-lang".to_string()),
                CssTokenKind::Delim('='),
                CssTokenKind::String("EN".to_string()),
                CssTokenKind::Ident("s".to_string()),
            ]
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-lang".to_string(), "EN".to_string());
        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    // === Misc Tests ===

    #[test]
    fn universal_selector() {
        let components = generate_compound_token!(CssTokenKind::Delim('*'));

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 0, 0));

        let tree = DocumentRoot::new();

        let hash_set = HashSet::new();

        let node_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), HashMap::default());

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn match_type_and_class_and_child_combinator() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("div".to_string()),
            CssTokenKind::Delim('.'),
            CssTokenKind::Ident("my-class".to_string()),
            CssTokenKind::Delim('>'),
            CssTokenKind::Ident("span".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 2);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 1, 1));
        assert_eq!(sequences[1].specificity(), SelectorSpecificity::new(0, 0, 1));

        let mut tree = DocumentRoot::new();

        let mut hash_set = HashSet::new();
        hash_set.insert("my-class".to_string());
        hash_set.insert("another-class".to_string());

        let parent_attributes = {
            let mut attrs = HashMap::new();
            attrs.insert("class".to_string(), "my-class another-class".to_string());
            attrs
        };
        let parent_data = generate_node_data!(HtmlTag::Div, hash_set.clone(), parent_attributes);
        let parent_id = tree.push_node(&parent_data, None);

        let child_data = generate_node_data!(HtmlTag::Span, HashSet::new(), HashMap::default());
        let child_id = tree.push_node(&child_data, Some(parent_id));

        let child_node = tree.get_node(&child_id).unwrap();

        assert!(matches_compound(&sequences, &tree, child_node, Some(&hash_set)));
    }

    #[test]
    fn match_child_combinator_with_surrounding_whitespace() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("div".to_string()),
            CssTokenKind::Whitespace,
            CssTokenKind::Delim('>'),
            CssTokenKind::Whitespace,
            CssTokenKind::Delim('.'),
            CssTokenKind::Ident("absolute".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 2);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(0, 0, 1));
        assert_eq!(sequences[1].specificity(), SelectorSpecificity::new(0, 1, 0));

        let mut tree = DocumentRoot::new();

        let outer_data = generate_node_data!(HtmlTag::Div, HashSet::new(), HashMap::default());
        let outer_id = tree.push_node(&outer_data, None);

        let middle_data = generate_node_data!(HtmlTag::Div, HashSet::new(), HashMap::default());
        let middle_id = tree.push_node(&middle_data, Some(outer_id));

        let mut absolute_classes = HashSet::new();
        absolute_classes.insert("absolute".to_string());
        let mut absolute_attributes = HashMap::new();
        absolute_attributes.insert("class".to_string(), "absolute".to_string());
        let absolute_data = generate_node_data!(HtmlTag::Div, absolute_classes.clone(), absolute_attributes);
        let absolute_id = tree.push_node(&absolute_data, Some(middle_id));

        let absolute_node = tree.get_node(&absolute_id).unwrap();

        assert!(matches_compound(&sequences, &tree, absolute_node, Some(&absolute_classes)));
    }

    #[test]
    fn match_very_specific_selector() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("a".to_string()),
            CssTokenKind::Delim('.'),
            CssTokenKind::Ident("external-link".to_string()),
            CssTokenKind::Hash {
                value: "main-link".to_string(),
                type_flag: HashType::Id,
            };
            attr[
                CssTokenKind::Ident("href".to_string()),
                CssTokenKind::Delim('^'),
                CssTokenKind::Delim('='),
                CssTokenKind::String("https://".to_string()),
            ],
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(1, 2, 1));

        let tree = DocumentRoot::new();

        let mut hash_set = HashSet::new();
        hash_set.insert("external-link".to_string());

        let mut attributes = HashMap::new();
        attributes.insert("class".to_string(), "external-link".to_string());
        attributes.insert("id".to_string(), "main-link".to_string());
        attributes.insert("href".to_string(), "https://example.com".to_string());

        let node_data = generate_node_data!(HtmlTag::A, hash_set.clone(), attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }

    #[test]
    fn empty_is_argument_does_not_generate_matchable_selector() {
        let components = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Colon,
                position: None,
            }),
            ComponentValue::Function(css_cssom::Function {
                name: "is".to_string(),
                value: vec![],
            }),
        ];

        let sequences = generate_compound_sequences(&components);

        assert!(sequences.is_empty());
    }

    #[test]
    fn descendant_chain_requires_all_steps() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("main".to_string()),
            CssTokenKind::Whitespace,
            CssTokenKind::Ident("section".to_string()),
            CssTokenKind::Whitespace,
            CssTokenKind::Ident("div".to_string())
        );

        let sequences = generate_compound_sequences(&components);

        let mut tree = DocumentRoot::new();

        let root_data = generate_node_data!(HtmlTag::Main, HashSet::new(), HashMap::default());
        let root_id = tree.push_node(&root_data, None);

        let middle_data = generate_node_data!(HtmlTag::Article, HashSet::new(), HashMap::default());
        let _middle_id = tree.push_node(&middle_data, Some(root_id));

        let target_data = generate_node_data!(HtmlTag::Div, HashSet::new(), HashMap::default());
        let target_id = tree.push_node(&target_data, Some(root_id));

        let target_node = tree.get_node(&target_id).unwrap();
        let classes = HashSet::new();

        assert!(!matches_compound(&sequences, &tree, target_node, Some(&classes)));
    }

    #[test]
    fn no_match_very_specific_selector() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("a".to_string()),
            CssTokenKind::Delim('.'),
            CssTokenKind::Ident("external-link".to_string()),
            CssTokenKind::Hash {
                value: "main-link".to_string(),
                type_flag: HashType::Id,
            };
            attr[
                CssTokenKind::Ident("href".to_string()),
                CssTokenKind::Delim('^'),
                CssTokenKind::Delim('='),
                CssTokenKind::String("https://".to_string()),
            ],
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].specificity(), SelectorSpecificity::new(1, 2, 1));

        let tree = DocumentRoot::new();

        let mut hash_set = HashSet::new();
        hash_set.insert("external-link".to_string());

        let mut attributes = HashMap::new();
        attributes.insert("class".to_string(), "external-link".to_string());
        attributes.insert("id".to_string(), "main-link".to_string());
        attributes.insert("href".to_string(), "http://example.com".to_string());

        let node_data = generate_node_data!(HtmlTag::A, hash_set.clone(), attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node, Some(&hash_set)));
    }
}
