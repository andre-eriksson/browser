use css_tokenizer::CssToken;

use crate::{
    AtRule, CssParser, QualifiedRule, Rule,
    consumers::{block::consume_simple_block, component::consume_component_value},
};

/// Consume a list of rules
///
/// <https://www.w3.org/TR/css-syntax-3/#consume-a-list-of-rules>
pub(crate) fn consume_list_of_rules(css_parser: &mut CssParser, top_level: bool) -> Vec<Rule> {
    let mut rules = Vec::new();

    loop {
        match css_parser.peek() {
            None => break,
            Some(CssToken::Eof) => break,
            Some(CssToken::Whitespace) => {
                css_parser.consume();
            }
            Some(CssToken::Cdo) | Some(CssToken::Cdc) => {
                if top_level {
                    css_parser.consume();
                } else if let Some(rule) = consume_qualified_rule(css_parser) {
                    rules.push(Rule::QualifiedRule(rule));
                }
            }
            Some(CssToken::AtKeyword(_)) => {
                rules.push(Rule::AtRule(consume_at_rule(css_parser)));
            }
            _ => {
                if let Some(rule) = consume_qualified_rule(css_parser) {
                    rules.push(Rule::QualifiedRule(rule));
                }
            }
        }
    }

    rules
}

/// Consume an at-rule
///
/// <https://www.w3.org/TR/css-syntax-3/#consume-an-at-rule>
pub(crate) fn consume_at_rule(css_parser: &mut CssParser) -> AtRule {
    // Consume the at-keyword token
    let name = match css_parser.consume() {
        Some(CssToken::AtKeyword(name)) => name,
        _ => String::new(), // Should not happen
    };

    let mut at_rule = AtRule::new(name);

    loop {
        match css_parser.peek() {
            None | Some(CssToken::Eof) => {
                // Parse error, but return the at-rule
                break;
            }
            Some(CssToken::Semicolon) => {
                css_parser.consume();
                break;
            }
            Some(CssToken::OpenCurly) => {
                at_rule.block = Some(consume_simple_block(css_parser));
                break;
            }
            _ => {
                at_rule.prelude.push(consume_component_value(css_parser));
            }
        }
    }

    at_rule
}

/// Consume a qualified rule
///
/// <https://www.w3.org/TR/css-syntax-3/#consume-a-qualified-rule>
fn consume_qualified_rule(css_parser: &mut CssParser) -> Option<QualifiedRule> {
    let mut rule = QualifiedRule::new();

    loop {
        match css_parser.peek() {
            None | Some(CssToken::Eof) => {
                // Parse error, return nothing
                return None;
            }
            Some(CssToken::OpenCurly) => {
                rule.block = consume_simple_block(css_parser);
                return Some(rule);
            }
            _ => {
                rule.prelude.push(consume_component_value(css_parser));
            }
        }
    }
}
