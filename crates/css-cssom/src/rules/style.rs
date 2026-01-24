use css_parser::{ComponentValue, QualifiedRule};
use serde::{Deserialize, Serialize};

use crate::{declaration::CSSDeclaration, rules::css::CSSRule, string::prelude_to_selector_text};

/// A CSS style rule (selector + declarations)
///
/// This is the interpreted form of a qualified rule at the top level of a stylesheet.
/// The prelude is interpreted as a selector list, and the block contains declarations.
///
/// <https://www.w3.org/TR/css-syntax-3/#style-rule>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CSSStyleRule {
    /// The selector text (prelude of the qualified rule)
    selector_text: String,

    /// The prelude as component values (for reference)
    pub prelude: Vec<ComponentValue>,

    /// The declarations in this style rule
    declarations: Vec<CSSDeclaration>,

    /// Nested rules (for CSS Nesting)
    pub nested_rules: Vec<CSSRule>,
}

impl CSSStyleRule {
    /// Create a new style rule with the given selector
    pub fn new(selector_text: String) -> Self {
        CSSStyleRule {
            selector_text,
            prelude: Vec::new(),
            declarations: Vec::new(),
            nested_rules: Vec::new(),
        }
    }

    /// Create a style rule from a parsed qualified rule
    ///
    /// Returns None if the prelude (selector) is empty/invalid.
    ///
    /// <https://www.w3.org/TR/css-syntax-3/#style-rule>
    pub fn from_parsed(qr: QualifiedRule, collect_positions: bool) -> Option<Self> {
        let selector_text = prelude_to_selector_text(&qr.prelude);
        if selector_text.is_empty() {
            return None;
        }

        let declarations = qr.parse_declarations(collect_positions);
        let css_declarations: Vec<CSSDeclaration> = declarations
            .into_iter()
            .map(CSSDeclaration::from_parser_declaration)
            .collect();

        let style_rule = CSSStyleRule {
            selector_text,
            prelude: qr.prelude,
            declarations: css_declarations,
            nested_rules: Vec::new(),
        };

        Some(style_rule)
    }

    /// Get the selector text
    pub fn selector_text(&self) -> &str {
        &self.selector_text
    }

    /// Get the declarations
    pub fn declarations(&self) -> &[CSSDeclaration] {
        &self.declarations
    }

    /// Get the value of a property by name
    pub fn get_property_value(&self, property: &str) -> Option<&str> {
        for decl in self.declarations.iter().rev() {
            if decl.name.eq_ignore_ascii_case(property) {
                return Some(&decl.value);
            }
        }
        None
    }

    /// Get the priority of a property by name (returns "important" or "")
    pub fn get_property_priority(&self, property: &str) -> &str {
        for decl in self.declarations.iter().rev() {
            if decl.name.eq_ignore_ascii_case(property) {
                return if decl.important { "important" } else { "" };
            }
        }
        ""
    }

    /// Set a property value
    pub fn set_property(&mut self, property: String, value: String, priority: bool) {
        self.declarations
            .retain(|d| !d.name.eq_ignore_ascii_case(&property));

        self.declarations.push(CSSDeclaration {
            name: property,
            value,
            important: priority,
            original_values: Vec::new(),
        });
    }

    /// Serialize this style rule to CSS text
    pub fn to_css_string(&self) -> String {
        let mut result = format!("{} {{\n", self.selector_text);

        for decl in &self.declarations {
            result.push_str("  ");
            result.push_str(&decl.to_css_string());
            result.push_str(";\n");
        }

        for nested in &self.nested_rules {
            result.push_str("  ");
            result.push_str(&nested.to_css_string().replace('\n', "\n  "));
            result.push('\n');
        }

        result.push('}');
        result
    }
}
