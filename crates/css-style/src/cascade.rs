use std::collections::HashMap;

use css_cssom::{CSSDeclaration, CSSStyleSheet, StylesheetOrigin};
use css_selectors::{SelectorSpecificity, SpecificityCalculable, matches_compound};
use html_dom::{DocumentRoot, DomNode};

use crate::cached_stylesheet::CachedStylesheets;

/// Full cascade specificity including inline styles
///
/// CSS Cascade ordering: (inline, id, class, element)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CascadeSpecificity(
    /// 1 if from inline style attribute, 0 otherwise
    pub u32,
    /// Number of ID selectors
    pub u32,
    /// Number of class/attribute selectors
    pub u32,
    /// Number of element selectors
    pub u32,
);

impl CascadeSpecificity {
    /// Create a CascadeSpecificity for inline styles
    pub fn inline() -> Self {
        CascadeSpecificity(1, 0, 0, 0)
    }
}

impl From<SelectorSpecificity> for CascadeSpecificity {
    fn from(spec: SelectorSpecificity) -> Self {
        CascadeSpecificity(0, spec.0, spec.1, spec.2)
    }
}

impl PartialOrd for CascadeSpecificity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CascadeSpecificity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .cmp(&other.0)
            .then(self.1.cmp(&other.1))
            .then(self.2.cmp(&other.2))
            .then(self.3.cmp(&other.3))
    }
}

#[derive(Debug, Clone)]
pub struct CascadedDeclaration {
    pub property: String,
    pub value: String,
    pub important: bool,
    pub specificity: CascadeSpecificity,
    pub source_order: usize,
    pub origin: StylesheetOrigin,
}

impl CascadedDeclaration {
    pub fn from_inline(decl: &CSSDeclaration, order: usize) -> Self {
        CascadedDeclaration {
            property: decl.name.clone(),
            value: decl.value.clone(),
            important: decl.important,
            specificity: CascadeSpecificity::inline(),
            source_order: order,
            origin: StylesheetOrigin::Author,
        }
    }
}

/// Collect declarations for a node using pre-cached selector sequences.
pub fn collect_declarations(
    node: &DomNode,
    dom: &DocumentRoot,
    cached_stylesheets: &CachedStylesheets,
) -> Vec<CascadedDeclaration> {
    let mut declarations = Vec::new();
    let mut source_order: usize = 0;

    let element = match node.data.as_element() {
        Some(elem) => elem,
        None => return declarations,
    };

    for cached_stylesheet in cached_stylesheets.iter() {
        for cached_rule in cached_stylesheet.cached_rules() {
            if matches_compound(&cached_rule.selector_sequences, dom, node) {
                let specificity = cached_rule
                    .selector_sequences
                    .iter()
                    .map(|seq| seq.specificity())
                    .max()
                    .unwrap_or_default();

                for decl in cached_rule.declarations {
                    let expanded = expand_shorthand_property(&decl.name, &decl.value);

                    for (property, value) in expanded {
                        declarations.push(CascadedDeclaration {
                            property,
                            value,
                            important: decl.important,
                            specificity: CascadeSpecificity::from(specificity),
                            source_order,
                            origin: cached_stylesheet.origin(),
                        });
                        source_order += 1;
                    }
                }
            }
        }
    }

    if let Some(style_attr) = element.get_attribute("style") {
        let inline_stylesheet =
            CSSStyleSheet::from_css(&format!("* {{ {} }}", style_attr), StylesheetOrigin::Author);
        for rule in inline_stylesheet.get_style_rules() {
            for decl in rule.declarations() {
                declarations.push(CascadedDeclaration::from_inline(decl, source_order));
                source_order += 1;
            }
        }
    }

    declarations
}

fn sort_declarations(declarations: &mut [CascadedDeclaration]) {
    declarations.sort_by(|a, b| {
        b.important
            .cmp(&a.important)
            .then_with(|| {
                let origin_order_a = match a.origin {
                    StylesheetOrigin::UserAgent => {
                        if a.important {
                            6
                        } else {
                            1
                        }
                    }
                    StylesheetOrigin::User => {
                        if a.important {
                            5
                        } else {
                            2
                        }
                    }
                    StylesheetOrigin::Author => {
                        if a.important {
                            4
                        } else {
                            3
                        }
                    }
                };
                let origin_order_b = match b.origin {
                    StylesheetOrigin::UserAgent => {
                        if b.important {
                            6
                        } else {
                            1
                        }
                    }
                    StylesheetOrigin::User => {
                        if b.important {
                            5
                        } else {
                            2
                        }
                    }
                    StylesheetOrigin::Author => {
                        if b.important {
                            4
                        } else {
                            3
                        }
                    }
                };
                origin_order_b.cmp(&origin_order_a)
            })
            .then_with(|| b.specificity.cmp(&a.specificity))
            .then_with(|| b.source_order.cmp(&a.source_order))
    });
}

fn expand_shorthand_property(property: &str, value: &str) -> Vec<(String, String)> {
    match property {
        "background" => vec![("background-color".to_string(), value.to_string())], // For now treat background as background-color
        _ => vec![(property.to_string(), value.to_string())],
    }
}

pub fn cascade(declarations: &mut [CascadedDeclaration]) -> HashMap<String, String> {
    sort_declarations(declarations);

    let mut cascaded_styles: HashMap<String, String> = HashMap::new();

    for decl in declarations.iter() {
        if !cascaded_styles.contains_key(&decl.property) {
            cascaded_styles.insert(decl.property.clone(), decl.value.clone());
        }
    }

    cascaded_styles
}
