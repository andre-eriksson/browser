use css_cssom::{AssociatedToken, ComponentValue, CssToken, CssTokenKind};
use html_syntax::dom::{DocumentRoot, DomNode, Element, NodeId};

/// The case sensitivity options for attribute selectors
#[derive(Debug, Clone, PartialEq, Default)]
pub enum CaseSensitivity {
    /// 's' or 'S' flag (default)
    #[default]
    CaseSensitive,

    /// 'i' or 'I' flag
    CaseInsensitive,
}

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

/// A CSS attribute selector
#[derive(Debug, Default)]
pub struct AttributeSelector {
    /// An `ident` token representing the attribute name
    attribute: Option<CssToken>,

    /// An optional operator for the attribute selector
    operator: Option<AttributeOperator>,

    /// An optional `ident` or `string` token representing the attribute value
    value: Option<CssToken>,

    /// An optional case sensitivity flag
    case: Option<CaseSensitivity>,
}

/// A compound selector consisting of simple selectors and attribute selectors
#[derive(Debug)]
pub struct CompoundSelector {
    /// A list of css tokens
    tokens: Vec<CssToken>,

    /// A list of attribute selectors
    attribute_selectors: Vec<AttributeSelector>,
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

/// A sequence of compound selectors with an optional combinator
#[derive(Debug)]
pub struct CompoundSelectorSequence {
    /// A list of compound selectors
    compound_selectors: Vec<CompoundSelector>,

    /// An optional combinator
    combinator: Option<Combinator>,
}

/// Parse an attribute selector from a list of CSS tokens
///
/// # Arguments
/// * `tokens` - A vector of CSS tokens representing the attribute selector
///
/// # Returns
/// * `Option<AttributeSelector>` - An optional AttributeSelector if parsing was successful
fn parse_attribute_selector(tokens: Vec<CssToken>) -> Option<AttributeSelector> {
    let mut attribute_selector = AttributeSelector::default();
    let mut temp_buffer = String::new();

    for token in tokens {
        if attribute_selector.attribute.is_none() {
            if let CssTokenKind::Ident(_) = token.kind {
                attribute_selector.attribute = Some(token);
                continue;
            } else {
                return None;
            }
        }

        if attribute_selector.operator.is_none() {
            if !temp_buffer.is_empty() {
                match temp_buffer.as_str() {
                    "~" => attribute_selector.operator = Some(AttributeOperator::Includes),
                    "|" => attribute_selector.operator = Some(AttributeOperator::DashMatch),
                    "^" => attribute_selector.operator = Some(AttributeOperator::PrefixMatch),
                    "$" => attribute_selector.operator = Some(AttributeOperator::SuffixMatch),
                    "*" => attribute_selector.operator = Some(AttributeOperator::SubstringMatch),
                    _ => return None,
                }
                temp_buffer.clear();
                continue;
            }

            match &token.kind {
                CssTokenKind::Delim('=') => {
                    attribute_selector.operator = Some(AttributeOperator::Equals);
                    continue;
                }
                CssTokenKind::Delim('~') => {
                    temp_buffer.push('~');
                    continue;
                }
                CssTokenKind::Delim('|') => {
                    temp_buffer.push('|');
                    continue;
                }
                CssTokenKind::Delim('^') => {
                    temp_buffer.push('^');
                    continue;
                }
                CssTokenKind::Delim('$') => {
                    temp_buffer.push('$');
                    continue;
                }
                CssTokenKind::Delim('*') => {
                    temp_buffer.push('*');
                    continue;
                }
                _ => {}
            }
        }

        if attribute_selector.value.is_none() {
            match &token.kind {
                CssTokenKind::Ident(_) | CssTokenKind::String(_) => {
                    attribute_selector.value = Some(token);
                    continue;
                }
                _ => {}
            }
        }

        if attribute_selector.case.is_none()
            && let CssTokenKind::Ident(ident) = &token.kind
        {
            match ident.as_str() {
                "i" | "I" => attribute_selector.case = Some(CaseSensitivity::CaseInsensitive),
                _ => attribute_selector.case = Some(CaseSensitivity::default()),
            }
        }
    }

    Some(attribute_selector)
}

/// Generate compound selector sequences from a list of component values
///
/// # Arguments
/// * `components` - A vector of ComponentValue representing the selector
///
/// # Returns
/// * `Vec<CompoundSelectorSequence>` - A vector of compound selector sequences
pub fn generate_compound_sequences(
    components: Vec<ComponentValue>,
) -> Vec<CompoundSelectorSequence> {
    let mut sequences: Vec<CompoundSelectorSequence> = Vec::new();
    let mut current_sequence = CompoundSelectorSequence {
        compound_selectors: Vec::new(),
        combinator: None,
    };

    for component in components.iter() {
        match component {
            ComponentValue::SimpleBlock(block) => {
                if block.associated_token == AssociatedToken::SquareBracket {
                    let attribute_selector = parse_attribute_selector(
                        block
                            .value
                            .iter()
                            .filter_map(|cv| {
                                if let ComponentValue::Token(t) = cv {
                                    Some(t.clone())
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    );

                    if let Some(attr_selector) = attribute_selector {
                        let compound_selector = current_sequence.compound_selectors.last_mut();
                        if let Some(cs) = compound_selector {
                            cs.attribute_selectors.push(attr_selector);
                        } else {
                            let new_compound_selector = CompoundSelector {
                                attribute_selectors: vec![attr_selector],
                                tokens: Vec::new(),
                            };

                            current_sequence
                                .compound_selectors
                                .push(new_compound_selector);
                        }
                    }
                }
            }
            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Delim('>') => {
                    current_sequence.combinator = Some(Combinator::Child);
                    sequences.push(current_sequence);
                    current_sequence = CompoundSelectorSequence {
                        compound_selectors: Vec::new(),
                        combinator: None,
                    };
                }
                CssTokenKind::Delim('+') => {
                    current_sequence.combinator = Some(Combinator::AdjacentSibling);
                    sequences.push(current_sequence);
                    current_sequence = CompoundSelectorSequence {
                        compound_selectors: Vec::new(),
                        combinator: None,
                    };
                }
                CssTokenKind::Delim('~') => {
                    current_sequence.combinator = Some(Combinator::GeneralSibling);
                    sequences.push(current_sequence);
                    current_sequence = CompoundSelectorSequence {
                        compound_selectors: Vec::new(),
                        combinator: None,
                    };
                }
                CssTokenKind::Whitespace => {
                    current_sequence.combinator = Some(Combinator::Descendant);
                    sequences.push(current_sequence);
                    current_sequence = CompoundSelectorSequence {
                        compound_selectors: Vec::new(),
                        combinator: None,
                    };
                }
                _ => {
                    let compound_selector = current_sequence.compound_selectors.last_mut();
                    if let Some(cs) = compound_selector {
                        cs.tokens.push(token.clone());
                    } else {
                        let new_compound_selector = CompoundSelector {
                            attribute_selectors: Vec::new(),
                            tokens: vec![token.clone()],
                        };

                        current_sequence
                            .compound_selectors
                            .push(new_compound_selector);
                    }
                }
            },
            _ => {}
        }
    }

    if !current_sequence.compound_selectors.is_empty() {
        sequences.push(current_sequence);
    }

    sequences
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

        if let CssTokenKind::Ident(ident) = &current_token.kind {
            let prev = previous_token.map(|t| &t.kind);
            let next = next_token.map(|t| &t.kind);

            if prev.is_none() || matches!(prev, Some(CssTokenKind::Whitespace)) {
                match next {
                    None | Some(CssTokenKind::Delim(_)) | Some(CssTokenKind::Whitespace) => {
                        if element.tag_name().to_string().to_lowercase() != ident.to_lowercase()
                            && ident != "*"
                        {
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

#[cfg(test)]
#[allow(clippy::vec_init_then_push)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use css_cssom::SimpleBlock;
    use html_syntax::{
        dom::NodeData,
        tag::{HtmlTag, KnownTag},
    };

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

        let sequences = generate_compound_sequences(components);

        assert_eq!(sequences.len(), 1);
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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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

        let sequences = generate_compound_sequences(components);

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
