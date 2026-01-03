use css_tokenizer::CssTokenKind;

use crate::{
    AssociatedToken, ComponentValue, CssParser, Declaration, DeclarationOrAtRule, SimpleBlock,
};

/// A CSS rule, either a qualified rule or an at-rule
///
/// <https://www.w3.org/TR/css-syntax-3/#css-rule>
#[derive(Debug, Clone, PartialEq)]
pub enum Rule {
    /// A qualified rule (typically a style rule)
    QualifiedRule(QualifiedRule),
    /// An at-rule (e.g., @media, @import)
    AtRule(AtRule),
}

/// A qualified rule has a prelude and a block
///
/// For style rules, the prelude is a selector and the block contains declarations.
///
/// <https://www.w3.org/TR/css-syntax-3/#qualified-rule>
#[derive(Debug, Clone, PartialEq)]
pub struct QualifiedRule {
    /// The prelude (typically a selector for style rules)
    pub prelude: Vec<ComponentValue>,
    /// The block containing declarations or nested rules
    pub block: SimpleBlock,
}

impl QualifiedRule {
    /// Create a new qualified rule
    pub fn new() -> Self {
        QualifiedRule {
            prelude: Vec::new(),
            block: SimpleBlock::new(AssociatedToken::CurlyBracket),
        }
    }

    /// Get the selector as a string representation
    ///
    /// This concatenates all the tokens in the prelude to form the selector string.
    pub fn selector_text(&self) -> String {
        let mut result = String::new();
        for cv in &self.prelude {
            result.push_str(&cv.to_css_string());
        }
        result.trim().to_string()
    }

    /// Parse the block contents as a list of declarations
    ///
    /// This is useful for style rules where the block contains property declarations.
    pub fn parse_declarations(&self) -> Vec<Declaration> {
        // Collect tokens from the block
        let mut tokens: Vec<CssTokenKind> = Vec::new();
        for cv in &self.block.value {
            Self::collect_tokens_from_component_value(cv, &mut tokens);
        }

        // Reconstruct input string and parse
        let input = tokens.iter().map(|t| t.to_string()).collect::<String>();

        let mut parser = CssParser::default();
        let decl_list = parser.parse_list_of_declarations(&input);

        decl_list
            .into_iter()
            .filter_map(|d| match d {
                DeclarationOrAtRule::Declaration(decl) => Some(decl),
                DeclarationOrAtRule::AtRule(_) => None,
            })
            .collect()
    }

    fn collect_tokens_from_component_value(cv: &ComponentValue, tokens: &mut Vec<CssTokenKind>) {
        match cv {
            ComponentValue::Token(t) => tokens.push(t.kind.clone()),
            ComponentValue::Function(f) => {
                tokens.push(CssTokenKind::Function(f.name.clone()));
                for v in &f.value {
                    Self::collect_tokens_from_component_value(v, tokens);
                }
                tokens.push(CssTokenKind::CloseParen);
            }
            ComponentValue::SimpleBlock(b) => {
                match b.associated_token {
                    AssociatedToken::CurlyBracket => tokens.push(CssTokenKind::OpenCurly),
                    AssociatedToken::SquareBracket => tokens.push(CssTokenKind::OpenSquare),
                    AssociatedToken::Parenthesis => tokens.push(CssTokenKind::OpenParen),
                }
                for v in &b.value {
                    Self::collect_tokens_from_component_value(v, tokens);
                }
                match b.associated_token {
                    AssociatedToken::CurlyBracket => tokens.push(CssTokenKind::CloseCurly),
                    AssociatedToken::SquareBracket => tokens.push(CssTokenKind::CloseSquare),
                    AssociatedToken::Parenthesis => tokens.push(CssTokenKind::CloseParen),
                }
            }
        }
    }
}

impl Default for QualifiedRule {
    fn default() -> Self {
        Self::new()
    }
}

/// An at-rule has a name, prelude, and optional block
///
/// <https://www.w3.org/TR/css-syntax-3/#at-rule>
#[derive(Debug, Clone, PartialEq)]
pub struct AtRule {
    /// The name of the at-rule (without the @)
    pub name: String,
    /// The prelude (everything between the name and the block/semicolon)
    pub prelude: Vec<ComponentValue>,
    /// The optional block (None if the at-rule ends with a semicolon)
    pub block: Option<SimpleBlock>,
}

impl AtRule {
    /// Create a new at-rule with the given name
    pub fn new(name: String) -> Self {
        AtRule {
            name,
            prelude: Vec::new(),
            block: None,
        }
    }
}
