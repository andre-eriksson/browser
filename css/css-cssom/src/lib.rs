//! CSS Stylesheet implementation following CSS Syntax Module Level 3
//!
//! This crate provides the CSS Stylesheet object model as defined in the
//! CSS Syntax Module Level 3 specification.
//!
//! <https://www.w3.org/TR/css-syntax-3/#css-stylesheets>
//!
//! # Overview
//!
//! A CSS stylesheet is a collection of CSS rules that define how elements
//! should be rendered. This module provides:
//!
//! - [`CSSStyleSheet`] - The main stylesheet object
//! - [`CSSRule`] - An enum representing different types of CSS rules
//! - [`CSSStyleRule`] - A style rule (selector + declarations)
//! - [`CSSAtRule`] - An at-rule (@media, @import, etc.)
//! - [`CSSDeclaration`] - A property declaration (name: value)
//!
//! # Example
//!
//! ```
//! use css_parser::CssParser;
//! use css_cssom::CSSStyleSheet;
//!
//! let mut parser = CssParser::default();
//! let parsed = parser.parse_css("div { color: red; }");
//! let stylesheet = CSSStyleSheet::from(parsed);
//! ```

pub mod cssom;
mod declaration;
mod rules;
mod string;

pub use css_parser::{ComponentValue, CssToken, CssTokenKind};
pub use cssom::CSSStyleSheet;
pub use declaration::CSSDeclaration;
pub use rules::at::CSSAtRule;
pub use rules::css::CSSRule;
pub use rules::style::CSSStyleRule;
