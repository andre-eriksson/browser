//! CSS Parser implementation following CSS Syntax Module Level 3
//!
//! This crate provides a CSS parser that follows the CSS Syntax Module Level 3 specification.
//! <https://www.w3.org/TR/css-syntax-3/#parsing>
//!
//! # Example
//!
//! ```
//! use css_parser::CssParser;
//!
//! let mut parser = CssParser::default();
//! let stylesheet = parser.parse_css("div { color: red; }");
//! ```

mod consumers;
mod parser;
mod rules;
mod stylesheet;

// Re-export main types for convenience
pub use parser::CssParser;
pub use rules::{AtRule, QualifiedRule, Rule};
pub use stylesheet::{
    AssociatedToken, ComponentValue, Declaration, DeclarationOrAtRule, Function, SimpleBlock,
    StyleBlockContents, Stylesheet,
};

pub use css_tokenizer::{CssToken, CssTokenKind, HashType, NumberType};
