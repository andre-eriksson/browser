use std::collections::{HashMap, HashSet};

use css_cssom::{CSSDeclaration, ComponentValue, CssTokenKind, HashType, Property, StylesheetOrigin};
use css_selectors::{ClassSet, CompoundSelectorSequence, SelectorSpecificity, matches_compound};

use html_dom::{DocumentRoot, DomNode, Element};

use crate::rules::GeneratedRule;

/// The key extracted from a rule's rightmost (subject) compound selector,
/// used for fast pre-filtering of rules that cannot possibly match an element.
enum SelectorKey {
    Id(String),
    Class(String),
    Tag(String),
    Universal,
}

/// Extracts the most selective key from the rightmost compound selector sequence.
fn extract_key_selector(sequences: &[CompoundSelectorSequence]) -> SelectorKey {
    let subject = match sequences.last() {
        Some(seq) => seq,
        None => return SelectorKey::Universal,
    };

    let mut id: Option<String> = None;
    let mut class: Option<String> = None;
    let mut tag: Option<String> = None;

    for compound in &subject.compound_selectors {
        let tokens = &compound.tokens;
        for i in 0..tokens.len() {
            let token = &tokens[i];
            match &token.kind {
                CssTokenKind::Hash { value, type_flag } if *type_flag == HashType::Id => {
                    if id.is_none() {
                        id = Some(value.to_ascii_lowercase());
                    }
                }
                CssTokenKind::Ident(name) => {
                    let prev = if i > 0 {
                        Some(&tokens[i - 1].kind)
                    } else {
                        None
                    };

                    if let Some(CssTokenKind::Delim('.')) = prev {
                        if class.is_none() {
                            class = Some(name.clone());
                        }
                    } else if prev.is_none() || matches!(prev, Some(CssTokenKind::Whitespace)) {
                        let next = tokens.get(i + 1).map(|t| &t.kind);
                        match next {
                            None | Some(CssTokenKind::Delim(_)) | Some(CssTokenKind::Whitespace) => {
                                if name != "*" && tag.is_none() {
                                    tag = Some(name.to_ascii_lowercase());
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(id) = id {
        SelectorKey::Id(id)
    } else if let Some(class) = class {
        SelectorKey::Class(class)
    } else if let Some(tag) = tag {
        SelectorKey::Tag(tag)
    } else {
        SelectorKey::Universal
    }
}

/// An index that groups rules by their rightmost selector key for fast lookup.
pub struct RuleIndex {
    by_id: HashMap<String, Vec<usize>>,
    by_class: HashMap<String, Vec<usize>>,
    by_tag: HashMap<String, Vec<usize>>,
    universal: Vec<usize>,
}

impl RuleIndex {
    /// Build a rule index from a slice of generated rules.
    pub fn build(rules: &[GeneratedRule]) -> Self {
        let mut by_id: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_class: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_tag: HashMap<String, Vec<usize>> = HashMap::new();
        let mut universal: Vec<usize> = Vec::new();

        for (idx, rule) in rules.iter().enumerate() {
            let key = extract_key_selector(&rule.selector_sequences);
            match key {
                SelectorKey::Id(id) => by_id.entry(id).or_default().push(idx),
                SelectorKey::Class(class) => by_class.entry(class).or_default().push(idx),
                SelectorKey::Tag(tag) => by_tag.entry(tag).or_default().push(idx),
                SelectorKey::Universal => universal.push(idx),
            }
        }

        Self {
            by_id,
            by_class,
            by_tag,
            universal,
        }
    }

    /// Return candidate rule indices for the given element, sorted by source order.
    ///
    /// This collects rules from all matching buckets (universal + tag + classes + id)
    /// and returns them sorted so that source_order assignment is consistent.
    fn candidates(&self, element: &Element) -> Vec<usize> {
        let mut candidates = Vec::with_capacity(self.universal.len() + 16);

        candidates.extend_from_slice(&self.universal);

        if let Some(indices) = self.by_tag.get(element.tag_name().as_str()) {
            candidates.extend_from_slice(indices);
        }

        for class in element.classes() {
            if let Some(indices) = self.by_class.get(class) {
                candidates.extend_from_slice(indices);
            }
        }

        if let Some(id) = element.id()
            && let Some(indices) = self.by_id.get(id)
        {
            candidates.extend_from_slice(indices);
        }

        candidates.sort_unstable();

        candidates
    }
}

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

/// A declaration that has been collected from the stylesheets and inline styles, along with its specificity and source order for cascade resolution.
#[derive(Debug, Clone)]
pub struct CascadedDeclaration<'rules> {
    pub property: &'rules Property,
    pub values: &'rules Vec<ComponentValue>,
    pub important: bool,
    pub specificity: CascadeSpecificity,
    pub source_order: usize,
    pub origin: StylesheetOrigin,
}

impl CascadedDeclaration<'_> {
    /// Collect all declarations that apply to the given DOM node from the provided stylesheets, including inline styles.
    pub fn collect<'css>(
        node: &DomNode,
        dom: &DocumentRoot,
        rules: &'css [GeneratedRule],
        rule_index: &RuleIndex,
        inline_declarations: &'css [CSSDeclaration],
    ) -> (Vec<CascadedDeclaration<'css>>, Vec<CascadedDeclaration<'css>>) {
        let mut declarations = Vec::new();
        let mut variables = Vec::new();
        let mut source_order: usize = 0;

        let element = match node.data.as_element() {
            Some(elem) => elem,
            None => return (vec![], vec![]),
        };

        let class_set = ClassSet::new(&element.class_set);
        let candidates = rule_index.candidates(element);

        for &idx in &candidates {
            let rule = &rules[idx];
            if matches_compound(&rule.selector_sequences, dom, node, &class_set) {
                for decl in rule.declarations {
                    if decl.property.is_custom() {
                        variables.push(CascadedDeclaration {
                            property: &decl.property,
                            values: &decl.original_values,
                            important: decl.important,
                            specificity: CascadeSpecificity::from(rule.specificity),
                            source_order,
                            origin: rule.origin,
                        });
                        source_order += 1;
                        continue;
                    }

                    declarations.push(CascadedDeclaration {
                        property: &decl.property,
                        values: &decl.original_values,
                        important: decl.important,
                        specificity: CascadeSpecificity::from(rule.specificity),
                        source_order,
                        origin: rule.origin,
                    });
                    source_order += 1;
                }
            }
        }

        for decl in inline_declarations {
            if decl.property.is_custom() {
                variables.push(CascadedDeclaration {
                    property: &decl.property,
                    values: &decl.original_values,
                    important: decl.important,
                    specificity: CascadeSpecificity::inline(),
                    source_order,
                    origin: StylesheetOrigin::Author,
                });
                source_order += 1;
                continue;
            }

            declarations.push(CascadedDeclaration {
                property: &decl.property,
                values: &decl.original_values,
                important: decl.important,
                specificity: CascadeSpecificity::inline(),
                source_order,
                origin: StylesheetOrigin::Author,
            });
            source_order += 1;
        }

        (declarations, variables)
    }

    fn origin_priority(origin: StylesheetOrigin, important: bool) -> u8 {
        match (origin, important) {
            (StylesheetOrigin::UserAgent, false) => 1,
            (StylesheetOrigin::User, false) => 2,
            (StylesheetOrigin::Author, false) => 3,
            (StylesheetOrigin::Author, true) => 4,
            (StylesheetOrigin::User, true) => 5,
            (StylesheetOrigin::UserAgent, true) => 6,
        }
    }

    /// Sort the declarations according to the CSS cascade rules: !important declarations first, then by origin (user agent, user, author),
    /// then by specificity, and finally by source order.
    fn sort_declarations(declarations: &mut [CascadedDeclaration]) {
        declarations.sort_by(|a, b| {
            b.important
                .cmp(&a.important)
                .then_with(|| {
                    Self::origin_priority(b.origin, b.important).cmp(&Self::origin_priority(a.origin, a.important))
                })
                .then_with(|| b.specificity.cmp(&a.specificity))
                .then_with(|| b.source_order.cmp(&a.source_order))
        });
    }
}

/// Perform the cascade and return the final set of properties and their values after applying all cascading rules, including inline styles and !important declarations.
pub fn cascade<'decl>(
    declarations: &'decl mut [CascadedDeclaration],
) -> Vec<(&'decl Property, &'decl Vec<ComponentValue>)> {
    CascadedDeclaration::sort_declarations(declarations);

    let mut cascaded_styles: Vec<(&Property, &Vec<ComponentValue>)> = Vec::with_capacity(32);
    let mut seen = HashSet::with_capacity(declarations.len());

    for decl in declarations.iter() {
        if seen.insert(decl.property) {
            cascaded_styles.push((decl.property, decl.values));
        }
    }

    cascaded_styles.reverse();
    cascaded_styles
}

/// Perform the cascade for custom properties (CSS variables) and return the final set of variables and their values after applying all cascading rules, including inline styles and !important declarations.
pub fn cascade_variables<'decl>(
    declarations: &'decl mut [CascadedDeclaration],
) -> HashMap<&'decl Property, &'decl Vec<ComponentValue>> {
    CascadedDeclaration::sort_declarations(declarations);

    let mut cascaded_variables: HashMap<&Property, &Vec<ComponentValue>> = HashMap::with_capacity(32);

    for decl in declarations.iter() {
        cascaded_variables
            .entry(decl.property)
            .or_insert(decl.values);
    }

    cascaded_variables
}
