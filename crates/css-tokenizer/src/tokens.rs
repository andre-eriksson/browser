use std::fmt::Display;

use crate::errors::SourcePosition;
use serde::{Deserialize, Serialize};

/// Hash token type flag as per CSS Syntax Module Level 3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashType {
    /// The hash token would start an identifier
    Id,
    /// The hash token would not start an identifier
    Unrestricted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum NumberType {
    Integer,
    Number,
}

/// A numeric value with its type flag
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NumericValue {
    /// The number is an integer (no decimal point or exponent)
    Integer(i64),
    /// The number has a decimal point and/or exponent
    Number(f64),
}

impl NumericValue {
    /// Get the numeric value as a f64, regardless of its original type
    #[must_use]
    pub const fn to_f64(&self) -> f64 {
        match self {
            Self::Integer(i) => *i as f64,
            Self::Number(n) => *n,
        }
    }

    /// Attempt to convert the numeric value to an i64 if it is an integer
    #[must_use]
    pub fn to_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            Self::Number(n) => {
                if !n.is_finite() {
                    return None;
                }

                if n.fract() == 0.0 && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 {
                    Some(*n as i64)
                } else {
                    None
                }
            }
        }
    }

    #[must_use]
    pub const fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    #[must_use]
    pub const fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }
}

impl From<f64> for NumericValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<i64> for NumericValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
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
            Self::Ident(value) => write!(f, "{value}",),
            Self::Function(value) => write!(f, "{value}("),
            Self::AtKeyword(value) => write!(f, "@{value}"),
            Self::Hash { value, .. } => write!(f, "#{value}"),
            Self::String(value) => write!(f, "\"{value}\""),
            Self::BadString => write!(f, "\""),
            Self::Url(value) => write!(f, "url({value})"),
            Self::BadUrl => write!(f, "url("),
            Self::Delim(c) => write!(f, "{c}"),
            Self::Number(num) => write!(f, "{}", num.to_f64()),
            Self::Percentage(num) => write!(f, "{}%", num.to_f64()),
            Self::Dimension { value, unit } => write!(f, "{}{unit}", value.to_f64()),
            Self::Whitespace => write!(f, " "),
            Self::Cdo => write!(f, "<!--"),
            Self::Cdc => write!(f, "-->"),
            Self::Colon => write!(f, ":"),
            Self::Semicolon => write!(f, ";"),
            Self::Comma => write!(f, ","),
            Self::OpenSquare => write!(f, "["),
            Self::CloseSquare => write!(f, "]"),
            Self::OpenParen => write!(f, "("),
            Self::CloseParen => write!(f, ")"),
            Self::OpenCurly => write!(f, "{{"),
            Self::CloseCurly => write!(f, "}}"),
            Self::Eof => Ok(()),
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
        Self {
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
