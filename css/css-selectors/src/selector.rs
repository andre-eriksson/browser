use css_cssom::{AssociatedToken, ComponentValue, CssToken, CssTokenKind};

use crate::{
    matching::{AttributeOperator, Combinator},
    parser::{CaseSensitivity, parse_attribute_selector},
};

/// A CSS attribute selector
#[derive(Debug, Default)]
pub struct AttributeSelector {
    /// An `ident` token representing the attribute name
    pub attribute: Option<CssToken>,

    /// An optional operator for the attribute selector
    pub operator: Option<AttributeOperator>,

    /// An optional `ident` or `string` token representing the attribute value
    pub value: Option<CssToken>,

    /// An optional case sensitivity flag
    pub case: Option<CaseSensitivity>,
}

/// A compound selector consisting of simple selectors and attribute selectors
#[derive(Debug)]
pub struct CompoundSelector {
    /// A list of css tokens
    pub tokens: Vec<CssToken>,

    /// A list of attribute selectors
    pub attribute_selectors: Vec<AttributeSelector>,
}

/// A sequence of compound selectors with an optional combinator
#[derive(Debug)]
pub struct CompoundSelectorSequence {
    /// A list of compound selectors
    pub compound_selectors: Vec<CompoundSelector>,

    /// An optional combinator
    pub combinator: Option<Combinator>,
}

/// Generate compound selector sequences from a list of component values
///
/// # Arguments
/// * `components` - A vector of ComponentValue representing the selector
///
/// # Returns
/// * `Vec<CompoundSelectorSequence>` - A vector of compound selector sequences
pub fn generate_compound_sequences(components: &[ComponentValue]) -> Vec<CompoundSelectorSequence> {
    let mut sequences: Vec<CompoundSelectorSequence> = Vec::new();
    let mut current_sequence = CompoundSelectorSequence {
        compound_selectors: Vec::new(),
        combinator: None,
    };

    let start = components
        .iter()
        .position(|cv| !cv.is_whitespace())
        .unwrap_or(components.len());
    let end = components
        .iter()
        .rposition(|cv| !cv.is_whitespace())
        .map(|idx| idx + 1)
        .unwrap_or(0);

    let trimmed_components = &components[start..end];

    for component in trimmed_components.iter() {
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
