//! CSS Stylesheet types following CSS Syntax Module Level 3
//!
//! <https://www.w3.org/TR/css-syntax-3/#css-stylesheets>

use css_parser::{CssParser, Stylesheet};
use serde::{Deserialize, Serialize};

use crate::rules::{css::CSSRule, style::CSSStyleRule};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StylesheetOrigin {
    /// Styles defined by the user-agent (browser default styles)
    UserAgent,

    /// Styles defined by the author of the document
    #[default]
    Author,

    /// Styles defined by the user, e.g., via browser settings
    User,
}

/// A CSS Stylesheet as defined in CSS Syntax Module Level 3
///
/// This represents the output of parsing a CSS stylesheet. It contains
/// a list of CSS rules and optional metadata such as location.
///
/// <https://www.w3.org/TR/css-syntax-3/#css-stylesheets>
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CSSStyleSheet {
    /// The list of CSS rules in this stylesheet
    rules: Vec<CSSRule>,

    /// The origin of this stylesheet (user-agent, user, author)
    origin: StylesheetOrigin,
}

impl CSSStyleSheet {
    pub fn from_css(css: &str, origin: StylesheetOrigin, collect_positions: bool) -> Self {
        let mut parser = CssParser::default();
        let parsed = parser.parse_css(css, collect_positions);

        let mut stylesheet = Self::from(parsed);
        stylesheet.origin = origin;
        stylesheet
    }

    pub fn origin(&self) -> StylesheetOrigin {
        self.origin
    }

    /// Get the list of CSS rules
    pub fn css_rules(&self) -> &[CSSRule] {
        &self.rules
    }

    /// Get the number of rules in this stylesheet
    pub fn length(&self) -> usize {
        self.rules.len()
    }

    /// Insert a rule at the specified index
    ///
    /// Returns an error if the index is out of bounds or the rule is invalid.
    pub fn insert_rule(&mut self, rule: CSSRule, index: usize) -> Result<usize, &'static str> {
        if index > self.rules.len() {
            return Err("Index out of bounds");
        }
        self.rules.insert(index, rule);
        Ok(index)
    }

    /// Delete the rule at the specified index
    ///
    /// Returns an error if the index is out of bounds.
    pub fn delete_rule(&mut self, index: usize) -> Result<(), &'static str> {
        if index >= self.rules.len() {
            return Err("Index out of bounds");
        }
        self.rules.remove(index);
        Ok(())
    }

    /// Serialize the stylesheet back to CSS text
    pub fn to_css_string(&self) -> String {
        let mut result = String::new();
        for rule in &self.rules {
            result.push_str(&rule.to_css_string());
            result.push('\n');
        }
        result
    }

    /// Get all style rules in this stylesheet (flattening nested rules)
    pub fn get_style_rules(&self) -> Vec<&CSSStyleRule> {
        let mut style_rules = Vec::new();
        for rule in &self.rules {
            Self::collect_style_rules(rule, &mut style_rules);
        }
        style_rules
    }

    fn collect_style_rules<'a>(rule: &'a CSSRule, collection: &mut Vec<&'a CSSStyleRule>) {
        match rule {
            CSSRule::Style(style_rule) => {
                collection.push(style_rule);
                for nested in &style_rule.nested_rules {
                    Self::collect_style_rules(nested, collection);
                }
            }
            CSSRule::AtRule(at_rule) => {
                for nested in &at_rule.rules {
                    Self::collect_style_rules(nested, collection);
                }
            }
        }
    }
}

impl From<Stylesheet> for CSSStyleSheet {
    fn from(parsed: Stylesheet) -> Self {
        let mut stylesheet = CSSStyleSheet::default();

        for rule in parsed.rules {
            if let Some(css_rule) = CSSRule::from_parsed(rule, true) {
                stylesheet.rules.push(css_rule);
            }
        }

        stylesheet
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use css_parser::CssParser;

    #[test]
    fn test_parse_simple_stylesheet() {
        let mut parser = CssParser::default();
        let parsed = parser.parse_css("div { color: red; }", true);
        let stylesheet = CSSStyleSheet::from(parsed);

        assert_eq!(stylesheet.length(), 1);
        assert!(stylesheet.css_rules()[0].is_style_rule());

        if let CSSRule::Style(style) = &stylesheet.css_rules()[0] {
            assert_eq!(style.selector_text(), "div");
            assert_eq!(style.declarations().len(), 1);
            assert_eq!(style.declarations()[0].name(), "color");
            assert_eq!(style.declarations()[0].value(), "red");
        }
    }

    #[test]
    fn test_parse_multiple_declarations() {
        let mut parser = CssParser::default();
        let parsed = parser.parse_css("p { margin: 10px; padding: 5px; }", true);
        let stylesheet = CSSStyleSheet::from(parsed);

        assert_eq!(stylesheet.length(), 1);

        if let CSSRule::Style(style) = &stylesheet.css_rules()[0] {
            assert_eq!(style.selector_text(), "p");
            assert_eq!(style.declarations().len(), 2);
            assert_eq!(style.get_property_value("margin"), Some("10px"));
            assert_eq!(style.get_property_value("padding"), Some("5px"));
        }
    }

    #[test]
    fn test_parse_important() {
        let mut parser = CssParser::default();
        let parsed = parser.parse_css("div { color: red !important; }", true);
        let stylesheet = CSSStyleSheet::from(parsed);

        if let CSSRule::Style(style) = &stylesheet.css_rules()[0] {
            assert!(style.declarations()[0].is_important());
            assert_eq!(style.get_property_priority("color"), "important");
        }
    }

    #[test]
    fn test_parse_at_rule_import() {
        let mut parser = CssParser::default();
        let parsed = parser.parse_css("@import url('styles.css');", true);
        let stylesheet = CSSStyleSheet::from(parsed);

        assert_eq!(stylesheet.length(), 1);
        assert!(stylesheet.css_rules()[0].is_at_rule());

        if let CSSRule::AtRule(at_rule) = &stylesheet.css_rules()[0] {
            assert_eq!(at_rule.name(), "import");
            assert!(!at_rule.has_block());
        }
    }

    #[test]
    fn test_parse_media_rule() {
        let mut parser = CssParser::default();
        let parsed = parser.parse_css("@media screen { div { color: blue; } }", true);
        let stylesheet = CSSStyleSheet::from(parsed);

        assert_eq!(stylesheet.length(), 1);

        if let CSSRule::AtRule(at_rule) = &stylesheet.css_rules()[0] {
            assert_eq!(at_rule.name(), "media");
            assert!(at_rule.has_block());
            assert_eq!(at_rule.prelude().trim(), "screen");
        }
    }

    #[test]
    fn test_parse_font_face() {
        let mut parser = CssParser::default();
        let parsed = parser.parse_css(
            "@font-face { font-family: 'MyFont'; src: url('font.woff2'); }",
            true,
        );
        let stylesheet = CSSStyleSheet::from(parsed);

        if let CSSRule::AtRule(at_rule) = &stylesheet.css_rules()[0] {
            assert_eq!(at_rule.name(), "font-face");
            assert!(at_rule.has_block());
            assert!(!at_rule.declarations().is_empty());
        }
    }

    #[test]
    fn test_stylesheet_serialization() {
        let mut parser = CssParser::default();
        let parsed = parser.parse_css("div { color: red; margin: 10px; }", true);
        let stylesheet = CSSStyleSheet::from(parsed);
        let css_text = stylesheet.to_css_string();

        assert!(css_text.contains("div"));
        assert!(css_text.contains("color: red"));
        assert!(css_text.contains("margin: 10px"));
    }

    #[test]
    fn test_insert_delete_rule() {
        let mut parser = CssParser::default();
        let parsed = parser.parse_css("div { color: red; }", true);
        let mut stylesheet = CSSStyleSheet::from(parsed);

        assert_eq!(stylesheet.length(), 1);

        // Create and insert a new rule
        let new_rule = CSSRule::Style(CSSStyleRule::new("p".to_string()));
        stylesheet.insert_rule(new_rule, 0).unwrap();
        assert_eq!(stylesheet.length(), 2);

        // Delete the first rule
        stylesheet.delete_rule(0).unwrap();
        assert_eq!(stylesheet.length(), 1);
    }

    #[test]
    fn test_style_rule_set_property() {
        let mut style_rule = CSSStyleRule::new("div".to_string());
        style_rule.set_property("color".to_string(), "blue".to_string(), false);

        assert_eq!(style_rule.get_property_value("color"), Some("blue"));
        assert_eq!(style_rule.get_property_priority("color"), "");

        style_rule.set_property("color".to_string(), "red".to_string(), true);
        assert_eq!(style_rule.get_property_value("color"), Some("red"));
        assert_eq!(style_rule.get_property_priority("color"), "important");
    }

    #[test]
    fn test_get_style_rules() {
        let mut parser = CssParser::default();
        let parsed = parser.parse_css(
            "div { color: red; } @media screen { p { color: blue; } } span { color: green; }",
            true,
        );
        let stylesheet = CSSStyleSheet::from(parsed);

        let style_rules = stylesheet.get_style_rules();
        // Should find div, p (inside @media), and span
        assert!(style_rules.len() >= 2);
    }
}
