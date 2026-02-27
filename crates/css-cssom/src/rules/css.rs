use css_parser::Rule;
use serde::{Deserialize, Serialize};

use crate::rules::{at::CSSAtRule, style::CSSStyleRule};

/// A CSS rule - either a style rule or an at-rule
///
/// <https://www.w3.org/TR/css-syntax-3/#css-rule>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub fn from_parsed(rule: Rule, collect_positions: bool) -> Option<Self> {
        match rule {
            Rule::QualifiedRule(qr) => CSSStyleRule::from_parsed(qr, collect_positions).map(CSSRule::Style),
            Rule::AtRule(ar) => Some(CSSRule::AtRule(CSSAtRule::from_parsed(ar, collect_positions))),
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

    /// Get this rule as a style rule, if it is one
    pub fn as_style_rule(&self) -> Option<&CSSStyleRule> {
        match self {
            CSSRule::Style(style) => Some(style),
            _ => None,
        }
    }

    /// Check if this is an at-rule
    pub fn is_at_rule(&self) -> bool {
        matches!(self, CSSRule::AtRule(_))
    }

    /// Get this rule as an at-rule, if it is one
    pub fn as_at_rule(&self) -> Option<&CSSAtRule> {
        match self {
            CSSRule::AtRule(at_rule) => Some(at_rule),
            _ => None,
        }
    }
}
