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
//! let stylesheet = parser.parse_css("div { color: red; }", false);
//! ```

mod consumers;
pub mod errors;
mod parser;
mod property;
mod rules;
mod stylesheet;

pub use css_tokenizer::{CssToken, CssTokenKind, HashType, NumericValue};
pub use parser::CssParser;
pub use property::{KnownProperty, Property};
pub use rules::{AtRule, QualifiedRule, Rule};
pub use stylesheet::{
    AssociatedToken, ComponentValue, Declaration, DeclarationOrAtRule, Function, SimpleBlock,
    StyleBlockContents, Stylesheet,
};
