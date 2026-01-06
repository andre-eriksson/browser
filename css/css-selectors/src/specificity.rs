use std::{cmp::Ordering, ops::Add};

use css_cssom::CssTokenKind;

use crate::selector::{CompoundSelector, CompoundSelectorSequence};

/// A CSS specificity value
///
/// (a, b, c) where:
/// - a: Number of ID selectors
/// - b: Number of class selectors, attributes selectors, and (TODO: pseudo-classes)
/// - c: Number of element selectors and (TODO: pseudo-elements)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Specificity(
    /// IDs
    pub u32,
    /// Classes, attributes, and (TODO: pseudo-classes)
    pub u32,
    /// Element selectors and (TODO: pseudo-elements)
    pub u32,
);

impl Specificity {
    pub fn new(a: u32, b: u32, c: u32) -> Self {
        Specificity(a, b, c)
    }
}

impl PartialOrd for Specificity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Specificity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .cmp(&other.0)
            .then(self.1.cmp(&other.1))
            .then(self.2.cmp(&other.2))
    }
}

impl Add for Specificity {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

/// A trait for calculating the specificity of CSS selectors
pub trait SpecificityCalculable {
    /// Calculate the specificity of the selector
    ///
    /// # Returns
    /// The calculated specificity
    fn specificity(&self) -> Specificity;
}

impl SpecificityCalculable for CompoundSelector {
    fn specificity(&self) -> Specificity {
        let mut specificity = Specificity::default();

        specificity.1 += self.attribute_selectors.len() as u32;

        for (i, token) in self.tokens.iter().enumerate() {
            if let CssTokenKind::Ident(ident) = &token.kind {
                let prev_token_kind = if i > 0 {
                    Some(&self.tokens[i - 1].kind)
                } else {
                    None
                };

                match prev_token_kind {
                    Some(CssTokenKind::Delim('#')) => {
                        specificity.0 += 1;
                    }
                    Some(CssTokenKind::Delim('.')) => {
                        specificity.1 += 1;
                    }
                    _ => {
                        if ident != "*" {
                            specificity.2 += 1;
                        }
                    }
                }
            }
        }

        specificity
    }
}

impl SpecificityCalculable for CompoundSelectorSequence {
    fn specificity(&self) -> Specificity {
        self.compound_selectors
            .iter()
            .map(|cs| cs.specificity())
            .fold(Specificity::default(), |acc, sp| acc + sp)
    }
}
