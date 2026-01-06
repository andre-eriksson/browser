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

// Re-export main types for convenience
pub use matching::{AttributeOperator, Combinator, matches_compound};
pub use parser::CaseSensitivity;
pub use selector::{
    AttributeSelector, CompoundSelector, CompoundSelectorSequence, generate_compound_sequences,
};
pub use specificity::{SelectorSpecificity, SpecificityCalculable};

#[cfg(test)]
#[allow(clippy::vec_init_then_push)]
mod tests {
    use std::collections::HashMap;

    use css_cssom::{AssociatedToken, ComponentValue, CssToken, CssTokenKind, SimpleBlock};
    use html_syntax::dom::{DocumentRoot, DomNode, Element, NodeData, NodeId};
    use html_syntax::tag::{HtmlTag, KnownTag};

    use crate::{SelectorSpecificity, SpecificityCalculable};
    use crate::{matching::matches_compound, selector::generate_compound_sequences};

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
        ($tag:expr, $attributes:expr) => {{ NodeData::Element(Element::new(HtmlTag::Known($tag), $attributes)) }};
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 1)
        );
        assert_eq!(sequences[0].compound_selectors.len(), 1);
        assert_eq!(sequences[0].compound_selectors[0].tokens.len(), 1);
        assert_eq!(
            sequences[0].compound_selectors[0].attribute_selectors.len(),
            1
        );
    }

    // === Basic Selector Tests ===

    #[test]
    fn match_type_selector() {
        let components = generate_compound_token!(CssTokenKind::Ident("div".to_string()));

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );

        let tree = DocumentRoot::new();

        let node_data = generate_node_data!(KnownTag::Div, HashMap::default());

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node));
    }

    #[test]
    fn no_match_type_selector() {
        let components = generate_compound_token!(CssTokenKind::Ident("span".to_string()));

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );

        let tree = DocumentRoot::new();

        let node_data = generate_node_data!(KnownTag::Div, HashMap::default());

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node));
    }

    #[test]
    fn match_class_selector() {
        let components = generate_compound_token!(
            CssTokenKind::Delim('.'),
            CssTokenKind::Ident("my-class".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("class".to_string(), "my-class another-class".to_string());

        let node_data = generate_node_data!(KnownTag::Div, attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node));
    }

    #[test]
    fn no_match_class_selector() {
        let components = generate_compound_token!(
            CssTokenKind::Delim('.'),
            CssTokenKind::Ident("my-class".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("class".to_string(), "wrong-class another-class".to_string());

        let node_data = generate_node_data!(KnownTag::Div, attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node));
    }

    #[test]
    fn match_id_selector() {
        let components = generate_compound_token!(
            CssTokenKind::Delim('#'),
            CssTokenKind::Ident("my-id".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(1, 0, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("id".to_string(), "my-id".to_string());

        let node_data = generate_node_data!(KnownTag::Div, attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node));
    }

    #[test]
    fn no_match_id_selector() {
        let components = generate_compound_token!(
            CssTokenKind::Delim('#'),
            CssTokenKind::Ident("my-id".to_string())
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(1, 0, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("id".to_string(), "wrong-id".to_string());

        let node_data = generate_node_data!(KnownTag::Div, attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 1)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("class".to_string(), "my-class another-class".to_string());

        let node_data = generate_node_data!(KnownTag::Div, attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 1)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("class".to_string(), "wrong-class another-class".to_string());

        let node_data = generate_node_data!(KnownTag::Div, attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );
        assert_eq!(
            sequences[1].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );

        let mut tree = DocumentRoot::new();

        let grandparent_data = generate_node_data!(KnownTag::Div, HashMap::default());
        let grandparent_id = tree.push_node(grandparent_data, None);

        let parent_data = generate_node_data!(KnownTag::Section, HashMap::default());
        let parent_id = tree.push_node(parent_data, Some(grandparent_id));

        let child_data = generate_node_data!(KnownTag::Span, HashMap::default());
        let child_id = tree.push_node(child_data, Some(parent_id));

        let child_node = tree.get_node(&child_id).unwrap();

        assert!(matches_compound(&sequences, &tree, child_node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );
        assert_eq!(
            sequences[1].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );

        let mut tree = DocumentRoot::new();

        let grandparent_data = generate_node_data!(KnownTag::Div, HashMap::default());
        let grandparent_id = tree.push_node(grandparent_data, None);

        let parent_data = generate_node_data!(KnownTag::Section, HashMap::default());
        let parent_id = tree.push_node(parent_data, Some(grandparent_id));

        let child_data = generate_node_data!(KnownTag::P, HashMap::default());
        let child_id = tree.push_node(child_data, Some(parent_id));

        let child_node = tree.get_node(&child_id).unwrap();

        assert!(!matches_compound(&sequences, &tree, child_node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );
        assert_eq!(
            sequences[1].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );

        let mut tree = DocumentRoot::new();

        let parent_data = generate_node_data!(KnownTag::Div, HashMap::default());
        let parent_id = tree.push_node(parent_data, None);

        let child_data = generate_node_data!(KnownTag::Span, HashMap::default());
        let child_id = tree.push_node(child_data, Some(parent_id));

        let child_node = tree.get_node(&child_id).unwrap();

        assert!(matches_compound(&sequences, &tree, child_node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );
        assert_eq!(
            sequences[1].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );

        let mut tree = DocumentRoot::new();

        let parent_data = generate_node_data!(KnownTag::Div, HashMap::default());
        let parent_id = tree.push_node(parent_data, None);

        let child_data = generate_node_data!(KnownTag::P, HashMap::default());
        let child_id = tree.push_node(child_data, Some(parent_id));

        let child_node = tree.get_node(&child_id).unwrap();

        assert!(!matches_compound(&sequences, &tree, child_node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );
        assert_eq!(
            sequences[1].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );

        let mut tree = DocumentRoot::new();

        let parent_data = generate_node_data!(KnownTag::Section, HashMap::default());
        let parent_id = tree.push_node(parent_data, None);

        let sibling1_data = generate_node_data!(KnownTag::Div, HashMap::default());
        tree.push_node(sibling1_data, Some(parent_id));

        let sibling2_data = generate_node_data!(KnownTag::Span, HashMap::default());
        let sibling2_id = tree.push_node(sibling2_data, Some(parent_id));

        let sibling2_node = tree.get_node(&sibling2_id).unwrap();

        assert!(matches_compound(&sequences, &tree, sibling2_node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );
        assert_eq!(
            sequences[1].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );

        let mut tree = DocumentRoot::new();

        let parent_data = generate_node_data!(KnownTag::Section, HashMap::default());
        let parent_id = tree.push_node(parent_data, None);

        let sibling1_data = generate_node_data!(KnownTag::Div, HashMap::default());
        tree.push_node(sibling1_data, Some(parent_id));

        let sibling2_data = generate_node_data!(KnownTag::P, HashMap::default());
        let sibling2_id = tree.push_node(sibling2_data, Some(parent_id));

        let sibling2_node = tree.get_node(&sibling2_id).unwrap();

        assert!(!matches_compound(&sequences, &tree, sibling2_node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );
        assert_eq!(
            sequences[1].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );

        let mut tree = DocumentRoot::new();

        let parent_data = generate_node_data!(KnownTag::Section, HashMap::default());
        let parent_id = tree.push_node(parent_data, None);

        let sibling1_data = generate_node_data!(KnownTag::Div, HashMap::default());
        tree.push_node(sibling1_data, Some(parent_id));

        let sibling2_data = generate_node_data!(KnownTag::P, HashMap::default());
        tree.push_node(sibling2_data, Some(parent_id));

        let sibling3_data = generate_node_data!(KnownTag::Span, HashMap::default());
        let sibling3_id = tree.push_node(sibling3_data, Some(parent_id));

        let sibling3_node = tree.get_node(&sibling3_id).unwrap();

        assert!(!matches_compound(&sequences, &tree, sibling3_node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );
        assert_eq!(
            sequences[1].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );

        let mut tree = DocumentRoot::new();

        let parent_data = generate_node_data!(KnownTag::Section, HashMap::default());
        let parent_id = tree.push_node(parent_data, None);

        let sibling1_data = generate_node_data!(KnownTag::Div, HashMap::default());
        tree.push_node(sibling1_data, Some(parent_id));

        let sibling2_data = generate_node_data!(KnownTag::Span, HashMap::default());
        let sibling2_id = tree.push_node(sibling2_data, Some(parent_id));

        let sibling2_node = tree.get_node(&sibling2_id).unwrap();

        assert!(matches_compound(&sequences, &tree, sibling2_node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );
        assert_eq!(
            sequences[1].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );

        let mut tree = DocumentRoot::new();

        let parent_data = generate_node_data!(KnownTag::Section, HashMap::default());
        let parent_id = tree.push_node(parent_data, None);

        let sibling1_data = generate_node_data!(KnownTag::Div, HashMap::default());
        tree.push_node(sibling1_data, Some(parent_id));

        let sibling2_data = generate_node_data!(KnownTag::P, HashMap::default());
        let sibling2_id = tree.push_node(sibling2_data, Some(parent_id));

        let sibling2_node = tree.get_node(&sibling2_id).unwrap();

        assert!(!matches_compound(&sequences, &tree, sibling2_node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );
        assert_eq!(
            sequences[1].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );

        let mut tree = DocumentRoot::new();

        let parent_data = generate_node_data!(KnownTag::Section, HashMap::default());
        let parent_id = tree.push_node(parent_data, None);

        let sibling1_data = generate_node_data!(KnownTag::Div, HashMap::default());
        tree.push_node(sibling1_data, Some(parent_id));

        let sibling2_data = generate_node_data!(KnownTag::P, HashMap::default());
        tree.push_node(sibling2_data, Some(parent_id));

        let sibling3_data = generate_node_data!(KnownTag::Span, HashMap::default());
        let sibling3_id = tree.push_node(sibling3_data, Some(parent_id));

        let sibling3_node = tree.get_node(&sibling3_id).unwrap();

        assert!(matches_compound(&sequences, &tree, sibling3_node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-active".to_string(), "true".to_string());

        let node_data = generate_node_data!(KnownTag::Div, attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-inactive".to_string(), "true".to_string());

        let node_data = generate_node_data!(KnownTag::Div, attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-test".to_string(), "value".to_string());

        let node_data = generate_node_data!(KnownTag::Div, attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("data".to_string(), "value".to_string());

        let node_data = generate_node_data!(KnownTag::Div, attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-tags".to_string(), "new featured popular".to_string());

        let node_data = generate_node_data!(KnownTag::Div, attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-tags".to_string(), "new discount popular".to_string());

        let node_data = generate_node_data!(KnownTag::Div, attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-region".to_string(), "us-west".to_string());
        let node_data = generate_node_data!(KnownTag::Div, attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-region".to_string(), "eu-north1".to_string());
        let node_data = generate_node_data!(KnownTag::Div, attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-lang".to_string(), "en-US".to_string());
        let node_data = generate_node_data!(KnownTag::Div, attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-lang".to_string(), "sv-SE".to_string());
        let node_data = generate_node_data!(KnownTag::Div, attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-file".to_string(), "image.jpg".to_string());
        let node_data = generate_node_data!(KnownTag::Div, attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-file".to_string(), "document.pdf".to_string());
        let node_data = generate_node_data!(KnownTag::Div, attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-info".to_string(), "start-middle-end".to_string());
        let node_data = generate_node_data!(KnownTag::Div, attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-info".to_string(), "start-end".to_string());
        let node_data = generate_node_data!(KnownTag::Div, attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-lang".to_string(), "en".to_string());
        let node_data = generate_node_data!(KnownTag::Div, attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 0)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("data-lang".to_string(), "EN".to_string());
        let node_data = generate_node_data!(KnownTag::Div, attributes);
        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node));
    }

    // === Misc Tests ===

    #[test]
    fn universal_selector() {
        let components = generate_compound_token!(CssTokenKind::Delim('*'));

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 0, 0)
        );

        let tree = DocumentRoot::new();

        let node_data = generate_node_data!(KnownTag::Div, HashMap::default());

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node));
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
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(0, 1, 1)
        );
        assert_eq!(
            sequences[1].specificity(),
            SelectorSpecificity::new(0, 0, 1)
        );

        let mut tree = DocumentRoot::new();

        let parent_attributes = {
            let mut attrs = HashMap::new();
            attrs.insert("class".to_string(), "my-class another-class".to_string());
            attrs
        };
        let parent_data = generate_node_data!(KnownTag::Div, parent_attributes);
        let parent_id = tree.push_node(parent_data, None);

        let child_data = generate_node_data!(KnownTag::Span, HashMap::default());
        let child_id = tree.push_node(child_data, Some(parent_id));

        let child_node = tree.get_node(&child_id).unwrap();

        assert!(matches_compound(&sequences, &tree, child_node));
    }

    #[test]
    fn match_very_specific_selector() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("a".to_string()),
            CssTokenKind::Delim('.'),
            CssTokenKind::Ident("external-link".to_string()),
            CssTokenKind::Delim('#'),
            CssTokenKind::Ident("main-link".to_string());
            attr[
                CssTokenKind::Ident("href".to_string()),
                CssTokenKind::Delim('^'),
                CssTokenKind::Delim('='),
                CssTokenKind::String("https://".to_string()),
            ],
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(1, 2, 1)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("class".to_string(), "external-link".to_string());
        attributes.insert("id".to_string(), "main-link".to_string());
        attributes.insert("href".to_string(), "https://example.com".to_string());

        let node_data = generate_node_data!(KnownTag::A, attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(matches_compound(&sequences, &tree, &node));
    }

    #[test]
    fn no_match_very_specific_selector() {
        let components = generate_compound_token!(
            CssTokenKind::Ident("a".to_string()),
            CssTokenKind::Delim('.'),
            CssTokenKind::Ident("external-link".to_string()),
            CssTokenKind::Delim('#'),
            CssTokenKind::Ident("main-link".to_string());
            attr[
                CssTokenKind::Ident("href".to_string()),
                CssTokenKind::Delim('^'),
                CssTokenKind::Delim('='),
                CssTokenKind::String("https://".to_string()),
            ],
        );

        let sequences = generate_compound_sequences(&components);
        assert_eq!(sequences.len(), 1);
        assert_eq!(
            sequences[0].specificity(),
            SelectorSpecificity::new(1, 2, 1)
        );

        let tree = DocumentRoot::new();

        let mut attributes = HashMap::new();
        attributes.insert("class".to_string(), "external-link".to_string());
        attributes.insert("id".to_string(), "main-link".to_string());
        attributes.insert("href".to_string(), "http://example.com".to_string());

        let node_data = generate_node_data!(KnownTag::A, attributes);

        let node = DomNode {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            data: node_data,
        };

        assert!(!matches_compound(&sequences, &tree, &node));
    }
}
