//! HTML Tokenizer Library
//!
//! This library provides a tokenizer for HTML content, breaking it down into manageable tokens for further processing.

/// Enum for parser states
mod state;

/// Various state handlers for the tokenizer
mod states;

/// The main tokenizer module
mod tokenizer;

/// Definitions for HTML tokens
mod tokens;

pub use state::TokenState;
pub use tokenizer::{HtmlTokenizer, TokenizerContext, TokenizerState};
pub use tokens::{Token, TokenKind};
