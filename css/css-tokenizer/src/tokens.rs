use std::fmt::Display;

/// Hash token type flag as per CSS Syntax Module Level 3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashType {
    /// The hash token would start an identifier
    Id,
    /// The hash token would not start an identifier
    Unrestricted,
}

/// Number type flag as per CSS Syntax Module Level 3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberType {
    /// The number is an integer (no decimal point or exponent)
    Integer,
    /// The number has a decimal point and/or exponent
    Number,
}

/// A numeric value with its type flag
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
pub enum CssToken {
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

impl Display for CssToken {
    /// Serialize the token to its CSS text representation
    ///
    /// This follows the CSS Syntax Module Level 3 serialization rules.
    ///<https://www.w3.org/TR/css-syntax-3/#serialization>
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CssToken::Ident(value) => write!(f, "{}", value),
            CssToken::Function(value) => write!(f, "{}(", value),
            CssToken::AtKeyword(value) => write!(f, "@{}", value),
            CssToken::Hash { value, .. } => write!(f, "#{}", value),
            CssToken::String(value) => write!(f, "\"{}\"", value),
            CssToken::BadString => write!(f, "\""),
            CssToken::Url(value) => write!(f, "url({})", value),
            CssToken::BadUrl => write!(f, "url("),
            CssToken::Delim(c) => write!(f, "{}", c),
            CssToken::Number(num) => write!(f, "{}", num.repr),
            CssToken::Percentage(num) => write!(f, "{}%", num.repr),
            CssToken::Dimension { value, unit } => write!(f, "{}{}", value.repr, unit),
            CssToken::Whitespace => write!(f, " "),
            CssToken::Cdo => write!(f, "<!--"),
            CssToken::Cdc => write!(f, "-->"),
            CssToken::Colon => write!(f, ":"),
            CssToken::Semicolon => write!(f, ";"),
            CssToken::Comma => write!(f, ","),
            CssToken::OpenSquare => write!(f, "["),
            CssToken::CloseSquare => write!(f, "]"),
            CssToken::OpenParen => write!(f, "("),
            CssToken::CloseParen => write!(f, ")"),
            CssToken::OpenCurly => write!(f, "{{"),
            CssToken::CloseCurly => write!(f, "}}"),
            CssToken::Eof => Ok(()),
        }
    }
}
