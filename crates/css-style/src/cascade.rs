use std::collections::HashMap;

use css_cssom::{
    CSSDeclaration, CSSRule, CSSStyleRule, CSSStyleSheet, ComponentValue, CssTokenKind, HashType,
    Property, StylesheetOrigin,
};
use css_selectors::{
    ClassSet, CompoundSelectorSequence, SelectorSpecificity, SpecificityCalculable,
    generate_selector_list, matches_compound,
};
use html_dom::{DocumentRoot, DomNode, Element};

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
                            None
                            | Some(CssTokenKind::Delim(_))
                            | Some(CssTokenKind::Whitespace) => {
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

        let tag = element.tag_name().to_ascii_lowercase();
        if let Some(indices) = self.by_tag.get(&tag) {
            candidates.extend_from_slice(indices);
        }

        for class in element.classes() {
            if let Some(indices) = self.by_class.get(class) {
                candidates.extend_from_slice(indices);
            }
        }

        if let Some(id) = element.id() {
            let id_lower = id.to_ascii_lowercase();
            if let Some(indices) = self.by_id.get(&id_lower) {
                candidates.extend_from_slice(indices);
            }
        }

        candidates.sort_unstable();

        candidates
    }
}

/// A rule that has been generated from the stylesheets, containing the selector sequences, declarations, origin, and specificity for cascade resolution.
#[derive(Debug)]
pub struct GeneratedRule<'a> {
    pub selector_sequences: Vec<CompoundSelectorSequence>,
    pub declarations: &'a [CSSDeclaration],
    pub origin: StylesheetOrigin,
    pub specificity: SelectorSpecificity,
}

impl<'a> GeneratedRule<'a> {
    pub fn build(stylesheets: &'a [CSSStyleSheet]) -> Vec<GeneratedRule<'a>> {
        let mut generated_rules = Vec::new();

        for stylesheet in stylesheets {
            for rule in stylesheet.css_rules() {
                match rule {
                    CSSRule::Style(style) => {
                        Self::push_rule(&mut generated_rules, stylesheet, style);
                    }
                    CSSRule::AtRule(at_rule) => {
                        if at_rule.name() == "import"
                            || at_rule.name() == "supports"
                            || at_rule.name() == "scope"
                            || at_rule.name() == "font-face"
                            || at_rule.name() == "keyframes"
                        {
                            continue;
                        }

                        if at_rule.prelude() == "print" {
                            continue;
                        }

                        for rule in &at_rule.rules {
                            if let Some(style_rule) = rule.as_style_rule() {
                                Self::push_rule(&mut generated_rules, stylesheet, style_rule);
                            }
                        }
                    }
                }
            }
        }

        generated_rules
    }

    fn push_rule(
        generated_rules: &mut Vec<GeneratedRule<'a>>,
        stylesheet: &CSSStyleSheet,
        style_rule: &'a CSSStyleRule,
    ) {
        let selector_list = generate_selector_list(&style_rule.prelude);
        for selector_sequence in selector_list {
            let specificity = selector_sequence
                .iter()
                .map(|seq| seq.specificity())
                .max()
                .unwrap_or_default();

            generated_rules.push(GeneratedRule {
                selector_sequences: selector_sequence,
                declarations: style_rule.declarations(),
                origin: stylesheet.origin(),
                specificity,
            });
        }
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
    pub fn collect<'a>(
        node: &DomNode,
        dom: &DocumentRoot,
        rules: &'a [GeneratedRule],
        rule_index: &RuleIndex,
        inline_declarations: &'a [CSSDeclaration],
    ) -> (Vec<CascadedDeclaration<'a>>, Vec<CascadedDeclaration<'a>>) {
        let mut declarations = Vec::new();
        let mut variables = Vec::new();
        let mut source_order: usize = 0;

        let element = match node.data.as_element() {
            Some(elem) => elem,
            None => return (vec![], vec![]),
        };

        let class_set = ClassSet::new(element.classes());
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

    /// Sort the declarations according to the CSS cascade rules: !important declarations first, then by origin (user agent, user, author),
    /// then by specificity, and finally by source order.
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
}

/// Perform the cascade and return the final set of properties and their values after applying all cascading rules, including inline styles and !important declarations.
pub fn cascade<'decl>(
    declarations: &'decl mut [CascadedDeclaration],
) -> Vec<(&'decl Property, &'decl Vec<ComponentValue>)> {
    CascadedDeclaration::sort_declarations(declarations);

    let mut cascaded_styles: Vec<(&Property, &Vec<ComponentValue>)> = Vec::with_capacity(32);

    for decl in declarations.iter() {
        if !cascaded_styles
            .iter()
            .any(|(prop, _)| prop == &decl.property)
        {
            cascaded_styles.push((decl.property, decl.values));
        }
    }

    cascaded_styles
}

/// Perform the cascade for custom properties (CSS variables) and return the final set of variables and their values after applying all cascading rules, including inline styles and !important declarations.
pub fn cascade_variables<'decl>(
    declarations: &'decl mut [CascadedDeclaration],
) -> HashMap<&'decl Property, &'decl Vec<ComponentValue>> {
    CascadedDeclaration::sort_declarations(declarations);

    let mut cascaded_variables: HashMap<&Property, &Vec<ComponentValue>> =
        HashMap::with_capacity(32);

    for decl in declarations.iter() {
        cascaded_variables
            .entry(decl.property)
            .or_insert(decl.values);
    }

    cascaded_variables
}
