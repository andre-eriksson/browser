//! CSS Parser implementation following CSS Syntax Module Level 3
//!
//! <https://www.w3.org/TR/css-syntax-3/#parsing>

use css_tokenizer::CssToken;
use css_tokenizer::CssTokenKind;
use css_tokenizer::CssTokenizer;
use css_tokenizer::SourcePosition;
use errors::parsing::CssParsingError;
use tracing::debug;

use crate::consumers::declaration::consume_list_of_declarations;
use crate::consumers::rule::consume_list_of_rules;
use crate::stylesheet::{
    AssociatedToken, ComponentValue, Declaration, DeclarationOrAtRule, Stylesheet,
};

/// CSS Parser following CSS Syntax Module Level 3
///
/// The parser transforms a stream of tokens into CSS objects such as
/// stylesheets, rules, and declarations.
#[derive(Debug, Clone, Default)]
pub struct CssParser {
    /// The list of tokens to parse
    tokens: Vec<CssToken>,

    /// Current position in the token list
    pos: usize,

    /// Collected parsing errors
    errors: Vec<CssParsingError>,
}

impl CssParser {
    pub(crate) fn new(tokens: Option<Vec<CssToken>>) -> Self {
        CssParser {
            tokens: tokens.unwrap_or_default(),
            pos: 0,
            errors: Vec::new(),
        }
    }

    /// Parse a stylesheet from a string
    ///
    /// <https://www.w3.org/TR/css-syntax-3/#parse-a-stylesheet>
    pub fn parse_css(&mut self, input: &str, collect_positions: bool) -> Stylesheet {
        self.tokens = CssTokenizer::tokenize(input, collect_positions);
        self.pos = 0;

        let rules = consume_list_of_rules(self, true);

        self.errors.sort_by_key(Self::get_error_pos);

        for error in &self.errors {
            debug!("CSS Parsing Error: {}", error);
        }

        Stylesheet { rules }
    }

    fn get_error_pos(error: &CssParsingError) -> SourcePosition {
        match error {
            CssParsingError::EofInAtRule(pos) => *pos,
            CssParsingError::EofInFunction(pos) => *pos,
            CssParsingError::IncompleteAtRule(pos) => *pos,
            CssParsingError::IncompleteFunction(pos) => *pos,
            CssParsingError::InvalidDeclarationStart(pos) => *pos,
            CssParsingError::IncompleteSimpleBlock(pos) => *pos,
            CssParsingError::IncompleteQualifiedRule(pos) => *pos,
            CssParsingError::EofInSimpleBlock(pos) => *pos,
            CssParsingError::EofInQualifiedRule(pos) => *pos,
            CssParsingError::EofInDeclaration(pos) => *pos,
            CssParsingError::InvalidDeclarationName(pos) => *pos,
            CssParsingError::MissingColonInDeclaration(pos) => *pos,
        }
    }

    pub fn record_error(&mut self, error: CssParsingError) {
        self.errors.push(error);
    }

    pub(crate) fn parse_stylesheet_from_tokens(&mut self, tokens: Vec<CssToken>) -> Stylesheet {
        self.tokens = tokens;
        self.pos = 0;

        let rules = consume_list_of_rules(self, true);

        Stylesheet { rules }
    }

    /// Parse a list of declarations
    ///
    /// <https://www.w3.org/TR/css-syntax-3/#parse-a-list-of-declarations>
    pub(crate) fn parse_list_of_declarations(
        &mut self,
        input: &str,
        collect_positions: bool,
    ) -> Vec<DeclarationOrAtRule> {
        self.tokens = CssTokenizer::tokenize(input, collect_positions);
        self.pos = 0;

        consume_list_of_declarations(self)
    }

    /// Check for !important and set the flag
    pub(crate) fn check_important(&self, declaration: &mut Declaration) {
        let mut non_ws_indices: Vec<usize> = Vec::new();

        for (i, cv) in declaration.value.iter().enumerate() {
            if !matches!(
                cv,
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Whitespace,
                    ..
                })
            ) {
                non_ws_indices.push(i);
            }
        }

        if non_ws_indices.len() >= 2 {
            let len = non_ws_indices.len();
            let second_last_idx = non_ws_indices[len - 2];
            let last_idx = non_ws_indices[len - 1];

            let is_important = matches!(
                (&declaration.value.get(second_last_idx), &declaration.value.get(last_idx)),
                (Some(ComponentValue::Token(CssToken { kind: CssTokenKind::Delim('!'), .. })),
                    Some(ComponentValue::Token(CssToken { kind: CssTokenKind::Ident(ident), .. })))
                if ident.eq_ignore_ascii_case("important")
            );

            if is_important {
                declaration.important = true;
                let mut indices_to_remove: Vec<usize> = Vec::new();

                for i in (second_last_idx..=declaration.value.len().saturating_sub(1)).rev() {
                    indices_to_remove.push(i);
                }

                for i in indices_to_remove {
                    if i < declaration.value.len() {
                        declaration.value.remove(i);
                    }
                }
            }
        }
    }

    /// Peek at the next token without consuming it
    pub(crate) fn peek(&self) -> Option<&CssToken> {
        self.tokens.get(self.pos)
    }

    /// Consume the next token
    pub(crate) fn consume(&mut self) -> Option<CssToken> {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }

    /// Check if we've reached the end of the token stream
    pub(crate) fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    /// Skip whitespace tokens
    pub(crate) fn skip_whitespace(&mut self) {
        while let Some(token) = self.peek() {
            if matches!(token.kind, CssTokenKind::Whitespace) {
                self.consume();
            } else {
                break;
            }
        }
    }

    /// Helper to append tokens from a component value
    pub(crate) fn append_component_value_tokens(cv: &ComponentValue, tokens: &mut Vec<CssToken>) {
        match cv {
            ComponentValue::Token(t) => tokens.push(t.clone()),
            ComponentValue::Function(f) => {
                tokens.push(CssToken {
                    kind: CssTokenKind::Function(f.name.clone()),
                    position: None,
                });
                for v in &f.value {
                    Self::append_component_value_tokens(v, tokens);
                }
                tokens.push(CssToken {
                    kind: CssTokenKind::CloseParen,
                    position: None,
                });
            }
            ComponentValue::SimpleBlock(b) => {
                match b.associated_token {
                    AssociatedToken::CurlyBracket => tokens.push(CssToken {
                        kind: CssTokenKind::OpenCurly,
                        position: None,
                    }),
                    AssociatedToken::SquareBracket => tokens.push(CssToken {
                        kind: CssTokenKind::OpenSquare,
                        position: None,
                    }),
                    AssociatedToken::Parenthesis => tokens.push(CssToken {
                        kind: CssTokenKind::OpenParen,
                        position: None,
                    }),
                }
                for v in &b.value {
                    Self::append_component_value_tokens(v, tokens);
                }
                match b.associated_token {
                    AssociatedToken::CurlyBracket => tokens.push(CssToken {
                        kind: CssTokenKind::CloseCurly,
                        position: None,
                    }),
                    AssociatedToken::SquareBracket => tokens.push(CssToken {
                        kind: CssTokenKind::CloseSquare,
                        position: None,
                    }),
                    AssociatedToken::Parenthesis => tokens.push(CssToken {
                        kind: CssTokenKind::CloseParen,
                        position: None,
                    }),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Rule;

    use super::*;

    #[test]
    fn test_parse_simple_stylesheet() {
        let mut parser = CssParser::default();
        let stylesheet = parser.parse_css("div { color: red; }", true);

        assert_eq!(stylesheet.rules.len(), 1);
        assert!(matches!(&stylesheet.rules[0], Rule::QualifiedRule(_)));
    }

    #[test]
    fn test_parse_at_rule() {
        let mut parser = CssParser::default();
        let stylesheet = parser.parse_css("@media print { body { font-size: 10pt } }", true);

        assert_eq!(stylesheet.rules.len(), 1);
        match &stylesheet.rules[0] {
            Rule::AtRule(at_rule) => {
                assert_eq!(at_rule.name, "media");
                assert!(at_rule.block.is_some());
            }
            _ => panic!("Expected at-rule"),
        }
    }

    #[test]
    fn test_parse_import_at_rule() {
        let mut parser = CssParser::default();
        let stylesheet = parser.parse_css("@import \"styles.css\";", true);

        assert_eq!(stylesheet.rules.len(), 1);
        match &stylesheet.rules[0] {
            Rule::AtRule(at_rule) => {
                assert_eq!(at_rule.name, "import");
                assert!(at_rule.block.is_none());
            }
            _ => panic!("Expected at-rule"),
        }
    }

    #[test]
    fn test_parse_multiple_rules() {
        let mut parser = CssParser::default();
        let stylesheet = parser.parse_css(
            "div { color: red; } p { margin: 0; } @media print { body { font-size: 10pt } }",
            true,
        );

        assert_eq!(stylesheet.rules.len(), 3);
    }

    #[test]
    fn test_parse_declaration_list() {
        let mut parser = CssParser::default();
        let declarations = parser.parse_list_of_declarations("color: red; margin: 10px", true);

        assert_eq!(declarations.len(), 2);
    }

    #[test]
    fn test_parse_important() {
        let mut parser = CssParser::default();
        let declarations = parser.parse_list_of_declarations("color: red !important", true);

        assert_eq!(declarations.len(), 1);
        match &declarations[0] {
            DeclarationOrAtRule::Declaration(decl) => {
                assert_eq!(decl.name, "color");
                assert!(decl.important);
            }
            _ => panic!("Expected declaration"),
        }
    }

    #[test]
    fn test_parse_function() {
        let mut parser = CssParser::default();
        let declarations = parser.parse_list_of_declarations("background: url(image.png)", true);

        assert_eq!(declarations.len(), 1);
        match &declarations[0] {
            DeclarationOrAtRule::Declaration(decl) => {
                assert_eq!(decl.name, "background");
                // Check that the value contains a function or URL token
                assert!(!decl.value.is_empty());
            }
            _ => panic!("Expected declaration"),
        }
    }

    #[test]
    fn test_parse_selector_text() {
        let mut parser = CssParser::default();
        let stylesheet = parser.parse_css("div.class#id > p:hover { color: red; }", true);

        assert_eq!(stylesheet.rules.len(), 1);
        match &stylesheet.rules[0] {
            Rule::QualifiedRule(qr) => {
                let selector = qr.selector_text();
                assert!(selector.contains("div"));
                assert!(selector.contains(".class"));
                assert!(selector.contains("#id"));
                assert!(selector.contains(">"));
                assert!(selector.contains("p"));
                assert!(selector.contains(":hover"));
            }
            _ => panic!("Expected qualified rule"),
        }
    }

    #[test]
    fn test_parse_declarations_from_block() {
        let mut parser = CssParser::default();
        let stylesheet = parser.parse_css("div { color: red; margin: 10px; padding: 5px; }", true);

        assert_eq!(stylesheet.rules.len(), 1);
        match &stylesheet.rules[0] {
            Rule::QualifiedRule(qr) => {
                let declarations = qr.parse_declarations(true);
                assert_eq!(declarations.len(), 3);
                assert_eq!(declarations[0].name, "color");
                assert_eq!(declarations[1].name, "margin");
                assert_eq!(declarations[2].name, "padding");
            }
            _ => panic!("Expected qualified rule"),
        }
    }

    #[test]
    fn test_parse_nested_blocks() {
        let mut parser = CssParser::default();
        let stylesheet = parser.parse_css(
            "@media screen and (min-width: 768px) { div { color: blue; } }",
            true,
        );

        assert_eq!(stylesheet.rules.len(), 1);
        match &stylesheet.rules[0] {
            Rule::AtRule(at_rule) => {
                assert_eq!(at_rule.name, "media");
                assert!(at_rule.block.is_some());
                // The block should contain the nested rule
                let block = at_rule.block.as_ref().unwrap();
                assert!(!block.value.is_empty());
            }
            _ => panic!("Expected at-rule"),
        }
    }

    #[test]
    fn test_parse_keyframes() {
        let mut parser = CssParser::default();
        let stylesheet = parser.parse_css(
            "@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }",
            true,
        );

        assert_eq!(stylesheet.rules.len(), 1);
        match &stylesheet.rules[0] {
            Rule::AtRule(at_rule) => {
                assert_eq!(at_rule.name, "keyframes");
                assert!(at_rule.block.is_some());
            }
            _ => panic!("Expected at-rule"),
        }
    }

    #[test]
    fn test_parse_font_face() {
        let mut parser = CssParser::default();
        let stylesheet = parser.parse_css(
            "@font-face { font-family: 'MyFont'; src: url('font.woff2'); }",
            true,
        );

        assert_eq!(stylesheet.rules.len(), 1);
        match &stylesheet.rules[0] {
            Rule::AtRule(at_rule) => {
                assert_eq!(at_rule.name, "font-face");
                assert!(at_rule.block.is_some());
            }
            _ => panic!("Expected at-rule"),
        }
    }

    #[test]
    fn test_parse_complex_selector() {
        let mut parser = CssParser::default();
        let stylesheet = parser.parse_css(
            "body > div.container p.text:first-child, header nav ul li a:hover { color: green; }",
            true,
        );

        assert_eq!(stylesheet.rules.len(), 1);
        match &stylesheet.rules[0] {
            Rule::QualifiedRule(qr) => {
                let selector = qr.selector_text();
                assert!(selector.contains("body"));
                assert!(selector.contains(","));
                assert!(selector.contains("header"));
            }
            _ => panic!("Expected qualified rule"),
        }
    }

    #[test]
    fn test_parse_calc_function() {
        let mut parser = CssParser::default();
        let declarations = parser.parse_list_of_declarations("width: calc(100% - 20px)", true);

        assert_eq!(declarations.len(), 1);
        match &declarations[0] {
            DeclarationOrAtRule::Declaration(decl) => {
                assert_eq!(decl.name, "width");
                // Value should contain a calc function
                let has_calc = decl.value.iter().any(|cv| {
                    matches!(cv, ComponentValue::Function(f) if f.name.eq_ignore_ascii_case("calc"))
                });
                assert!(has_calc);
            }
            _ => panic!("Expected declaration"),
        }
    }

    #[test]
    fn test_parse_rgb_function() {
        let mut parser = CssParser::default();
        let declarations = parser.parse_list_of_declarations("color: rgb(255, 128, 0)", true);

        assert_eq!(declarations.len(), 1);
        match &declarations[0] {
            DeclarationOrAtRule::Declaration(decl) => {
                assert_eq!(decl.name, "color");
                // Value should contain rgb function
                let has_rgb = decl.value.iter().any(|cv| {
                    matches!(cv, ComponentValue::Function(f) if f.name.eq_ignore_ascii_case("rgb"))
                });
                assert!(has_rgb);
            }
            _ => panic!("Expected declaration"),
        }
    }

    #[test]
    fn test_parse_empty_stylesheet() {
        let mut parser = CssParser::default();
        let stylesheet = parser.parse_css("", true);

        assert!(stylesheet.rules.is_empty());
    }

    #[test]
    fn test_parse_whitespace_only_stylesheet() {
        let mut parser = CssParser::default();
        let stylesheet = parser.parse_css("   \n\t  ", true);

        assert!(stylesheet.rules.is_empty());
    }

    #[test]
    fn test_parse_comments_handled_by_tokenizer() {
        let mut parser = CssParser::default();
        let stylesheet = parser.parse_css(
            "/* comment */
 div { color: red; }",
            true,
        );

        assert_eq!(stylesheet.rules.len(), 1);
    }

    #[test]
    fn test_parse_multiple_declarations_same_property() {
        let mut parser = CssParser::default();
        let declarations =
            parser.parse_list_of_declarations("color: red; color: blue; color: green", true);

        assert_eq!(declarations.len(), 3);
    }

    #[test]
    fn test_parse_declaration_with_fallback() {
        let mut parser = CssParser::default();
        let declarations = parser.parse_list_of_declarations(
            "background: red; background: linear-gradient(to right, red, blue)",
            true,
        );

        assert_eq!(declarations.len(), 2);
    }

    #[test]
    fn test_parse_var_function() {
        let mut parser = CssParser::default();
        let declarations =
            parser.parse_list_of_declarations("color: var(--primary-color, blue)", true);

        assert_eq!(declarations.len(), 1);
        match &declarations[0] {
            DeclarationOrAtRule::Declaration(decl) => {
                assert_eq!(decl.name, "color");
                // Value should contain var function
                let has_var = decl
                    .value
                    .iter()
                    .any(|cv| matches!(cv, ComponentValue::Function(f) if f.name == "var"));
                assert!(has_var);
            }
            _ => panic!("Expected declaration"),
        }
    }

    #[test]
    fn test_parse_custom_property() {
        let mut parser = CssParser::default();
        let declarations = parser.parse_list_of_declarations("--primary-color: #ff0000", true);

        assert_eq!(declarations.len(), 1);
        match &declarations[0] {
            DeclarationOrAtRule::Declaration(decl) => {
                assert_eq!(decl.name, "--primary-color");
            }
            _ => panic!("Expected declaration"),
        }
    }
}
