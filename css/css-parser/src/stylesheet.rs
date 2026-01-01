use css_tokenizer::CssToken;

use crate::{
    CssParser,
    rules::{AtRule, Rule},
};

/// A CSS stylesheet containing a list of rules
///
/// <https://www.w3.org/TR/css-syntax-3/#css-stylesheet>
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
    /// The property name
    pub name: String,
    /// The value as a list of component values
    pub value: Vec<ComponentValue>,
    /// Whether this declaration has !important
    pub important: bool,
}

impl Declaration {
    /// Create a new declaration
    pub fn new(name: String) -> Self {
        Declaration {
            name,
            value: Vec::new(),
            important: false,
        }
    }
}

/// A component value is a preserved token, function, or simple block
///
/// <https://www.w3.org/TR/css-syntax-3/#component-value>
#[derive(Debug, Clone, PartialEq)]
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
    pub fn to_css_string(&self) -> String {
        match self {
            ComponentValue::Token(t) => t.to_string(),
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
}

/// A CSS function (name followed by parentheses with content)
///
/// <https://www.w3.org/TR/css-syntax-3/#function>
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq)]
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
