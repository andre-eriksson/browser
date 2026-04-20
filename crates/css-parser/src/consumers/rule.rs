use crate::errors::CssParsingError;
use css_tokenizer::{CssTokenKind, SourcePosition};

use crate::{
    AtRule, CssParser, QualifiedRule, Rule,
    consumers::{block::consume_simple_block, component::consume_component_value},
};

/// Consume a list of rules
///
/// <https://www.w3.org/TR/css-syntax-3/#consume-a-list-of-rules>
pub fn consume_list_of_rules(css_parser: &mut CssParser, top_level: bool) -> Vec<Rule> {
    let mut rules = Vec::new();

    while let Some(token) = css_parser.peek() {
        match &token.kind {
            CssTokenKind::Eof => break,
            CssTokenKind::Whitespace => {
                css_parser.consume();
            }
            CssTokenKind::Cdo | CssTokenKind::Cdc => {
                if top_level {
                    css_parser.consume();
                } else if let Some(rule) = consume_qualified_rule(css_parser) {
                    rules.push(Rule::QualifiedRule(rule));
                }
            }
            CssTokenKind::AtKeyword(_) => {
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
pub fn consume_at_rule(css_parser: &mut CssParser) -> AtRule {
    let (name, pos) = match css_parser.consume() {
        Some(token) => {
            let pos = token.position.unwrap_or_default();

            match token.kind {
                CssTokenKind::AtKeyword(name) => (name, pos),
                _ => {
                    unreachable!("consume_at_rule should only be called when the current token is an at-keyword token")
                }
            }
        }
        None => unreachable!("consume_at_rule should only be called when there is a current token"),
    };

    let mut at_rule = AtRule::new(name);

    #[allow(clippy::while_let_loop)]
    loop {
        if let Some(token) = css_parser.peek() {
            match &token.kind {
                CssTokenKind::Eof => {
                    css_parser.record_error(CssParsingError::EofInAtRule(pos));
                    break;
                }
                CssTokenKind::Semicolon => {
                    css_parser.consume();
                    break;
                }
                CssTokenKind::OpenCurly => {
                    at_rule.block = Some(consume_simple_block(css_parser));
                    break;
                }
                _ => {
                    at_rule.prelude.push(consume_component_value(css_parser));
                }
            }
        } else {
            css_parser.record_error(CssParsingError::IncompleteAtRule(pos));
            break;
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
        if let Some(token) = css_parser.peek() {
            match &token.kind {
                CssTokenKind::Eof => {
                    let pos = token.position.unwrap_or_default();
                    css_parser.record_error(CssParsingError::EofInQualifiedRule(pos));
                    return None;
                }
                CssTokenKind::OpenCurly => {
                    rule.block = consume_simple_block(css_parser);
                    return Some(rule);
                }
                _ => {
                    rule.prelude.push(consume_component_value(css_parser));
                }
            }
        } else {
            // Parse error, return nothing
            css_parser.record_error(CssParsingError::IncompleteQualifiedRule(SourcePosition::default()));
            return None;
        }
    }
}
