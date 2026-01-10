//! HTML Tokenizer Library

/// Enum for parser states
mod state;

/// Various state handlers for the tokenizer
mod states;

/// The main tokenizer module
mod tokenizer;

mod tokens;

pub use state::TokenState;
pub use tokenizer::{HtmlTokenizer, TokenizerContext, TokenizerState};
pub use tokens::{Token, TokenKind};
