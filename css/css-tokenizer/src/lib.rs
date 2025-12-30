//! CSS Tokenizer implementation following CSS Syntax Module Level 3
//!
//! This crate provides a CSS tokenizer that follows the CSS Syntax Module Level 3 specification.
//! <https://www.w3.org/TR/css-syntax-3/#tokenization>
//!
//! # Example
//!
//! ```
//! use css_tokenizer::{CssTokenizer, CssToken};
//!
//! let mut tokenizer = CssTokenizer::new("div { color: red; }");
//! let tokens: Vec<CssToken> = tokenizer.tokenize();
//! for token in tokens {
//!     println!("{:?}", token);
//! }
//! ```

/// Utilities for character classification defined in §4.2 of the CSS Syntax Module Level 3
///
/// <https://www.w3.org/TR/css-syntax-3/#tokenizer-definitions>
mod char;

/// Consumers for different CSS tokens as defined in the CSS Syntax Module Level 3
///
/// <https://www.w3.org/TR/css-syntax-3/#tokenizer-algorithms>
mod consumers;

/// The main tokenizer implementation
pub mod tokenizer;

/// Definitions of CSS tokens as per the CSS Syntax Module Level 3
pub mod tokens;

/// Validation utilities for ensuring compliance with CSS Syntax Module Level 3
mod validator;
