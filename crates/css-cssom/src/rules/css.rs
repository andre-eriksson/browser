use std::fmt::Display;

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
    /// Create a `CSSRule` from a parsed Rule
    ///
    /// Returns None if the rule is invalid and should be discarded.
    pub fn from_parsed(rule: Rule, collect_positions: bool) -> Option<Self> {
        match rule {
            Rule::QualifiedRule(qr) => CSSStyleRule::from_parsed(qr, collect_positions).map(CSSRule::Style),
            Rule::AtRule(ar) => Some(Self::AtRule(CSSAtRule::from_parsed(ar, collect_positions))),
        }
    }

    /// Serialize this rule to CSS text
    fn to_css_string(&self) -> String {
        match self {
            Self::Style(style) => style.to_string(),
            Self::AtRule(at_rule) => at_rule.to_string(),
        }
    }

    /// Check if this is a style rule
    #[must_use]
    pub const fn is_style_rule(&self) -> bool {
        matches!(self, Self::Style(_))
    }

    /// Get this rule as a style rule, if it is one
    #[must_use]
    pub const fn as_style_rule(&self) -> Option<&CSSStyleRule> {
        match self {
            Self::Style(style) => Some(style),
            Self::AtRule(_) => None,
        }
    }

    /// Check if this is an at-rule
    #[must_use]
    pub const fn is_at_rule(&self) -> bool {
        matches!(self, Self::AtRule(_))
    }

    /// Get this rule as an at-rule, if it is one
    #[must_use]
    pub const fn as_at_rule(&self) -> Option<&CSSAtRule> {
        match self {
            Self::AtRule(at_rule) => Some(at_rule),
            Self::Style(_) => None,
        }
    }
}

impl Display for CSSRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_css_string())
    }
}
