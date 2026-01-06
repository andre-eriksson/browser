use std::collections::HashMap;

use css_cssom::{CSSDeclaration, CSSStyleSheet, StylesheetOrigin};
use css_selectors::{
    SelectorSpecificity, SpecificityCalculable, generate_compound_sequences, matches_compound,
};
use html_dom::{DocumentRoot, DomNode};

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
}

impl CascadedDeclaration {
    pub fn from_inline(decl: &CSSDeclaration, order: usize) -> Self {
        CascadedDeclaration {
            property: decl.name.clone(),
            value: decl.value.clone(),
            important: decl.important,
            specificity: CascadeSpecificity::inline(),
            source_order: order,
        }
    }
}

pub fn collect_declarations(
    node: &DomNode,
    dom: &DocumentRoot,
    stylesheets: &[CSSStyleSheet],
) -> Vec<CascadedDeclaration> {
    let mut declarations = Vec::new();
    let mut source_order: usize = 0;

    let element = match node.data.as_element() {
        Some(elem) => elem,
        None => return declarations,
    };

    for stylesheet in stylesheets {
        for rule in stylesheet.get_style_rules() {
            let selector_sequences = generate_compound_sequences(&rule.prelude);

            if matches_compound(&selector_sequences, dom, node) {
                let specificity = selector_sequences
                    .iter()
                    .map(|seq| seq.specificity())
                    .max()
                    .unwrap_or_default();

                for decl in rule.declarations() {
                    declarations.push(CascadedDeclaration {
                        property: decl.name.clone(),
                        value: decl.value.clone(),
                        important: decl.important,
                        specificity: CascadeSpecificity::from(specificity),
                        source_order,
                    });
                    source_order += 1;
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

pub fn cascade(declarations: &mut [CascadedDeclaration]) -> HashMap<String, String> {
    declarations.sort_by(|a, b| {
        a.important
            .cmp(&b.important)
            .then(a.specificity.cmp(&b.specificity))
            .then(a.source_order.cmp(&b.source_order))
    });

    let mut cascaded_styles: HashMap<String, String> = HashMap::new();

    for decl in declarations.iter() {
        if !cascaded_styles.contains_key(&decl.property) {
            cascaded_styles.insert(decl.property.clone(), decl.value.clone());
        }
    }

    cascaded_styles
}
