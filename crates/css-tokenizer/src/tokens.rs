use std::fmt::Display;

use errors::tokenization::SourcePosition;
use serde::{Deserialize, Serialize};

/// Hash token type flag as per CSS Syntax Module Level 3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashType {
    /// The hash token would start an identifier
    Id,
    /// The hash token would not start an identifier
    Unrestricted,
}

/// Number type flag as per CSS Syntax Module Level 3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberType {
    /// The number is an integer (no decimal point or exponent)
    Integer,
    /// The number has a decimal point and/or exponent
    Number,
}

/// A numeric value with its type flag
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumericValue {
    /// The numeric value
    pub value: f64,
    /// The integer value (if type is Integer)
    pub int_value: Option<i64>,
    /// Whether this is an integer or number
    pub type_flag: NumberType,
    /// The original representation in the source
    pub repr: String,
}

impl NumericValue {
    /// Create a new NumericValue
    ///
    /// # Arguments
    /// * `value` - The numeric value as f64
    /// * `repr` - The original string representation
    /// * `type_flag` - The type flag indicating if it's an integer or number
    pub fn new(value: f64, repr: String, type_flag: NumberType) -> Self {
        let int_value = if type_flag == NumberType::Integer {
            Some(value as i64)
        } else {
            None
        };
        Self {
            value,
            int_value,
            type_flag,
            repr,
        }
    }
}

/// CSS Token as per CSS Syntax Module Level 3
/// <https://www.w3.org/TR/css-syntax-3/#tokenization>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CssTokenKind {
    /// \<ident-token\>: An identifier
    Ident(String),

    ///\<function-token\>: A function (name followed by '(')
    Function(String),

    ///\<at-keyword-token\>: An at-keyword (e.g., @media)
    AtKeyword(String),

    ///\<hash-token\>: A hash (e.g., #fff or #id)
    Hash { value: String, type_flag: HashType },

    ///\<string-token\>: A quoted string
    String(String),

    ///\<bad-string-token\>: An invalid string (e.g., contains unescaped newline)
    BadString,

    ///\<url-token\>: A URL token (url(...) with unquoted content)
    Url(String),

    ///\<bad-url-token\>: An invalid URL token
    BadUrl,

    ///\<delim-token\>: A single code point not consumed by any other token
    Delim(char),

    ///\<number-token\>: A numeric value
    Number(NumericValue),

    ///\<percentage-token\>: A percentage value
    Percentage(NumericValue),

    ///\<dimension-token\>: A number with a unit
    Dimension { value: NumericValue, unit: String },

    ///\<whitespace-token\>: One or more whitespace characters
    Whitespace,

    ///\<CDO-token\>: \<!--
    Cdo,

    ///\<CDC-token\>: --\>
    Cdc,

    ///\<colon-token\>: :
    Colon,

    ///\<semicolon-token\>: ;
    Semicolon,

    ///\<comma-token\>: ,
    Comma,

    ///\<[-token\>: [
    OpenSquare,

    ///\<]-token\>: ]
    CloseSquare,

    ///\<(-token\>: (
    OpenParen,

    ///\<)-token\>: )
    CloseParen,

    ///\<{-token\>: {
    OpenCurly,

    ///\<}-token\>: }
    CloseCurly,

    /// End of file marker (not emitted, used internally)
    Eof,
}

impl Display for CssTokenKind {
    /// Serialize the token to its CSS text representation
    ///
    /// This follows the CSS Syntax Module Level 3 serialization rules.
    ///<https://www.w3.org/TR/css-syntax-3/#serialization>
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CssTokenKind::Ident(value) => write!(f, "{}", value),
            CssTokenKind::Function(value) => write!(f, "{}(", value),
            CssTokenKind::AtKeyword(value) => write!(f, "@{}", value),
            CssTokenKind::Hash { value, .. } => write!(f, "#{}", value),
            CssTokenKind::String(value) => write!(f, "\"{}\"", value),
            CssTokenKind::BadString => write!(f, "\""),
            CssTokenKind::Url(value) => write!(f, "url({})", value),
            CssTokenKind::BadUrl => write!(f, "url("),
            CssTokenKind::Delim(c) => write!(f, "{}", c),
            CssTokenKind::Number(num) => write!(f, "{}", num.repr),
            CssTokenKind::Percentage(num) => write!(f, "{}%", num.repr),
            CssTokenKind::Dimension { value, unit } => write!(f, "{}{}", value.repr, unit),
            CssTokenKind::Whitespace => write!(f, " "),
            CssTokenKind::Cdo => write!(f, "<!--"),
            CssTokenKind::Cdc => write!(f, "-->"),
            CssTokenKind::Colon => write!(f, ":"),
            CssTokenKind::Semicolon => write!(f, ";"),
            CssTokenKind::Comma => write!(f, ","),
            CssTokenKind::OpenSquare => write!(f, "["),
            CssTokenKind::CloseSquare => write!(f, "]"),
            CssTokenKind::OpenParen => write!(f, "("),
            CssTokenKind::CloseParen => write!(f, ")"),
            CssTokenKind::OpenCurly => write!(f, "{{"),
            CssTokenKind::CloseCurly => write!(f, "}}"),
            CssTokenKind::Eof => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CssToken {
    pub kind: CssTokenKind,
    pub position: Option<SourcePosition>,
}

impl From<CssTokenKind> for CssToken {
    fn from(kind: CssTokenKind) -> Self {
        CssToken {
            kind,
            position: None,
        }
    }
}

impl Display for CssToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}
