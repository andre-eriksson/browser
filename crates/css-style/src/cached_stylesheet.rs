use css_cssom::{CSSDeclaration, CSSStyleSheet, StylesheetOrigin};
use css_selectors::{CompoundSelectorSequence, generate_selector_list};

/// A cached style rule containing pre-computed selector sequences
#[derive(Debug)]
pub struct CachedStyleRule<'a> {
    /// Pre-computed selector sequences for this rule
    pub selector_sequences: Vec<CompoundSelectorSequence>,

    /// Reference to the original declarations
    pub declarations: &'a [CSSDeclaration],
}

/// A wrapper around `CSSStyleSheet` that caches parsed selector sequences.
#[derive(Debug)]
pub struct CachedStylesheet<'a> {
    /// The underlying stylesheet
    stylesheet: &'a CSSStyleSheet,

    /// Cached style rules with pre-computed selectors
    cached_rules: Vec<CachedStyleRule<'a>>,
}

impl<'a> CachedStylesheet<'a> {
    /// Create a new cached stylesheet from a `CSSStyleSheet`.
    ///
    /// This pre-computes all selector sequences for style rules in the stylesheet.
    pub fn new(stylesheet: &'a CSSStyleSheet) -> Self {
        let style_rules = stylesheet.get_style_rules();

        let mut cached_rules = Vec::new();
        for rule in style_rules {
            let selector_list = generate_selector_list(&rule.prelude);
            for selector_sequence in selector_list {
                cached_rules.push(CachedStyleRule {
                    selector_sequences: selector_sequence,
                    declarations: rule.declarations(),
                });
            }
        }

        CachedStylesheet {
            stylesheet,
            cached_rules,
        }
    }

    /// Get the cached style rules with their pre-computed selector sequences.
    pub fn cached_rules(&self) -> &[CachedStyleRule<'a>] {
        &self.cached_rules
    }

    /// Get the origin of the underlying stylesheet.
    pub fn origin(&self) -> StylesheetOrigin {
        self.stylesheet.origin()
    }
}

/// A collection of cached stylesheets for efficient style computation.
///
/// This is typically created once per style tree build and reused for all nodes.
#[derive(Debug)]
pub struct CachedStylesheets<'a> {
    /// The cached stylesheets
    stylesheets: Vec<CachedStylesheet<'a>>,
}

impl<'a> CachedStylesheets<'a> {
    /// Create a new collection of cached stylesheets.
    pub fn new(stylesheets: &'a [CSSStyleSheet]) -> Self {
        let cached = stylesheets.iter().map(CachedStylesheet::new).collect();

        CachedStylesheets {
            stylesheets: cached,
        }
    }

    /// Get an iterator over the cached stylesheets.
    pub fn iter(&self) -> impl Iterator<Item = &CachedStylesheet<'a>> {
        self.stylesheets.iter()
    }
}
