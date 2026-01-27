//! HTML DOM module
//!
//! This module provides structures and utilities for working with the HTML Document Object Model (DOM).

/// The DOM tree builder.
mod builder;

/// The Collector for gathering information during parsing, like tags and attributes.
///
/// This is optional and can be customized.
mod collector;

/// The HTML decoder module.
///
/// This module provides utilities for decoding HTML entities.
mod decode;

/// DOM based structures and utilities.
mod dom;

/// HTML tags and related utilities.
mod tag;

pub use builder::{BuildResult, DomTreeBuilder};
pub use collector::{Collector, DefaultCollector, TagInfo};
pub use dom::{DocumentRoot, DomNode, Element, NodeData, NodeId};
pub use html_tokenizer::{HtmlTokenizer, Token, TokenState, TokenizerState};
pub use tag::{HtmlTag, Tag};
