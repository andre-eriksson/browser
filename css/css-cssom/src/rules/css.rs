use css_parser::Rule;

use crate::rules::{at::CSSAtRule, style::CSSStyleRule};

/// A CSS rule - either a style rule or an at-rule
///
/// <https://www.w3.org/TR/css-syntax-3/#css-rule>
#[derive(Debug, Clone, PartialEq)]
pub enum CSSRule {
    /// A style rule (qualified rule interpreted as a style rule)
    Style(CSSStyleRule),

    /// An at-rule (@media, @import, @font-face, etc.)
    AtRule(CSSAtRule),
}

impl CSSRule {
    /// Create a CSSRule from a parsed Rule
    ///
    /// Returns None if the rule is invalid and should be discarded.
    pub fn from_parsed(rule: Rule) -> Option<Self> {
        match rule {
            Rule::QualifiedRule(qr) => CSSStyleRule::from_parsed(qr).map(CSSRule::Style),
            Rule::AtRule(ar) => Some(CSSRule::AtRule(CSSAtRule::from_parsed(ar))),
        }
    }

    /// Serialize this rule to CSS text
    pub fn to_css_string(&self) -> String {
        match self {
            CSSRule::Style(style) => style.to_css_string(),
            CSSRule::AtRule(at_rule) => at_rule.to_css_string(),
        }
    }

    /// Check if this is a style rule
    pub fn is_style_rule(&self) -> bool {
        matches!(self, CSSRule::Style(_))
    }

    /// Check if this is an at-rule
    pub fn is_at_rule(&self) -> bool {
        matches!(self, CSSRule::AtRule(_))
    }
}
