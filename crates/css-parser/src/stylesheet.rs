use std::fmt::Display;

use css_tokenizer::CssToken;
use serde::{Deserialize, Serialize};

use crate::{
    CssParser,
    property::Property,
    rules::{AtRule, Rule},
};

/// A CSS stylesheet containing a list of rules
///
/// <https://www.w3.org/TR/css-syntax-3/#parsing>
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Stylesheet {
    /// The list of rules in the stylesheet
    pub rules: Vec<Rule>,
}

impl From<Vec<CssToken>> for Stylesheet {
    fn from(tokens: Vec<CssToken>) -> Self {
        let mut parser = CssParser::default();
        parser.parse_stylesheet_from_tokens(tokens)
    }
}

/// A CSS declaration (property: value)
///
/// <https://www.w3.org/TR/css-syntax-3/#declaration>
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    /// The property type (known or custom)
    pub property: Property,
    /// The value as a list of component values
    pub value: Vec<ComponentValue>,
    /// Whether this declaration has !important
    pub important: bool,
}

impl Declaration {
    /// Create a new declaration
    pub fn new(property: Property) -> Self {
        Declaration {
            property,
            value: Vec::new(),
            important: false,
        }
    }
}

/// A component value is a preserved token, function, or simple block
///
/// <https://www.w3.org/TR/css-syntax-3/#component-value>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComponentValue {
    /// A preserved token
    Token(CssToken),
    /// A function
    Function(Function),
    /// A simple block
    SimpleBlock(SimpleBlock),
}

impl ComponentValue {
    /// Convert this component value to a CSS string representation
    fn to_css_string(&self) -> String {
        match self {
            ComponentValue::Token(t) => t.kind.to_string(),
            ComponentValue::Function(f) => {
                let mut s = format!("{}(", f.name);
                for v in &f.value {
                    s.push_str(&v.to_css_string());
                }
                s.push(')');
                s
            }
            ComponentValue::SimpleBlock(b) => {
                let (open, close) = match b.associated_token {
                    AssociatedToken::CurlyBracket => ('{', '}'),
                    AssociatedToken::SquareBracket => ('[', ']'),
                    AssociatedToken::Parenthesis => ('(', ')'),
                };
                let mut s = String::new();
                s.push(open);
                for v in &b.value {
                    s.push_str(&v.to_css_string());
                }
                s.push(close);
                s
            }
        }
    }

    /// Check if this component value is a token
    pub fn is_token(&self) -> bool {
        matches!(self, ComponentValue::Token(_))
    }

    /// Get a reference to the token if this component value is a token
    pub fn as_token(&self) -> Option<&CssToken> {
        match self {
            ComponentValue::Token(t) => Some(t),
            _ => None,
        }
    }

    /// Check if this component value is a whitespace token
    ///
    /// # Returns
    /// True if it is a whitespace token, false otherwise
    pub fn is_whitespace(&self) -> bool {
        match self {
            ComponentValue::Token(t) => matches!(t.kind, css_tokenizer::CssTokenKind::Whitespace),
            _ => false,
        }
    }
}

impl Display for ComponentValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_css_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentValueStream<'a> {
    values: &'a [ComponentValue],
    position: usize,
}

impl<'a> ComponentValueStream<'a> {
    pub fn new(values: &'a [ComponentValue]) -> ComponentValueStream<'a> {
        ComponentValueStream {
            values,
            position: 0,
        }
    }

    /// Get the underlying slice of component values
    pub fn values(&self) -> &[ComponentValue] {
        self.values
    }

    pub fn position(&self) -> usize {
        self.position
    }

    /// Peek at the next component value without consuming it
    pub fn peek(&self) -> Option<&ComponentValue> {
        self.values.get(self.position)
    }

    /// Consume and return the next component value
    pub fn next_cv(&mut self) -> Option<&ComponentValue> {
        let value = self.values.get(self.position);
        if value.is_some() {
            self.position += 1;
        }
        value
    }

    /// Create a checkpoint of the current position in the stream
    pub fn checkpoint(&self) -> usize {
        self.position
    }

    /// Restore the stream to a previously created checkpoint
    pub fn restore(&mut self, checkpoint: usize) {
        self.position = checkpoint;
    }

    /// Returns the unconsumed portion of the underlying slice.
    pub fn remaining(&self) -> &[ComponentValue] {
        &self.values[self.position..]
    }

    /// Skip over any whitespace tokens in the stream
    pub fn skip_whitespace(&mut self) {
        while let Some(value) = self.peek() {
            if value.is_whitespace() {
                self.next_cv();
            } else {
                break;
            }
        }
    }

    /// Skip whitespace and return the next non-whitespace component value
    pub fn next_non_whitespace(&mut self) -> Option<&ComponentValue> {
        self.skip_whitespace();
        self.next_cv()
    }

    pub fn split_by<F>(self, mut predicate: F) -> impl Iterator<Item = ComponentValueStream<'a>>
    where
        F: FnMut(&ComponentValue) -> bool,
    {
        self.values
            .split(move |item| predicate(item))
            .map(move |sub_slice| ComponentValueStream {
                position: 0,
                values: sub_slice,
            })
    }
}

impl<'a> From<&'a [ComponentValue]> for ComponentValueStream<'a> {
    fn from(values: &'a [ComponentValue]) -> Self {
        ComponentValueStream::new(values)
    }
}

impl<'a> From<&'a Vec<ComponentValue>> for ComponentValueStream<'a> {
    fn from(values: &'a Vec<ComponentValue>) -> Self {
        ComponentValueStream::new(values)
    }
}

/// A CSS function (name followed by parentheses with content)
///
/// <https://www.w3.org/TR/css-syntax-3/#function>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    /// The function name
    pub name: String,
    /// The function's value (content between parentheses)
    pub value: Vec<ComponentValue>,
}

impl Function {
    /// Create a new function with the given name
    pub fn new(name: String) -> Self {
        Function {
            name,
            value: Vec::new(),
        }
    }
}

/// The associated token type for a simple block
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssociatedToken {
    /// { } block
    CurlyBracket,
    /// [ ] block
    SquareBracket,
    /// ( ) block
    Parenthesis,
}

/// A simple block has an associated token and a value
///
/// <https://www.w3.org/TR/css-syntax-3/#simple-block>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimpleBlock {
    /// The associated token (determines the block type)
    pub associated_token: AssociatedToken,
    /// The block's value (content between brackets)
    pub value: Vec<ComponentValue>,
}

impl SimpleBlock {
    /// Create a new simple block with the given associated token
    pub fn new(associated_token: AssociatedToken) -> Self {
        SimpleBlock {
            associated_token,
            value: Vec::new(),
        }
    }
}

/// Result of parsing a style block's contents
///
/// Contains both declarations and nested rules.
#[derive(Debug, Clone, PartialEq)]
pub struct StyleBlockContents {
    /// Declarations in the style block
    pub declarations: Vec<DeclarationOrAtRule>,
    /// Nested rules in the style block
    pub rules: Vec<Rule>,
}

impl StyleBlockContents {
    /// Create new empty style block contents
    pub fn new() -> Self {
        StyleBlockContents {
            declarations: Vec::new(),
            rules: Vec::new(),
        }
    }
}

impl Default for StyleBlockContents {
    fn default() -> Self {
        Self::new()
    }
}

/// Either a declaration or an at-rule (used in declaration lists)
#[derive(Debug, Clone, PartialEq)]
pub enum DeclarationOrAtRule {
    /// A declaration
    Declaration(Declaration),
    /// An at-rule
    AtRule(AtRule),
}
