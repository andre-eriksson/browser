//! HTML parsing library for Rust.
//!
//! This library is the **core** HTML parser used in the browser project. It provides
//! streaming HTML parsing capabilities, allowing for efficient processing of HTML content
//! in a non-blocking manner. The parser can handle scripts and styles by pausing
//! execution until the necessary resources are available.
//!
//! # Flow of the HTML Parser
//! 1. The `HtmlStreamParser` reads HTML content from a stream.
//! 2. It tokenizes the content using the `html_tokenizer` crate.
//! 3. The parser processes tokens and builds a DOM-like structure.
//! 4. If a `<script>` or `<style>` tag is encountered, the parser blocks and waits for the content to be provided.
//! 5. Once the content is available, the parser resumes processing.
//! 6. The parser continues until the entire HTML content is processed.

/// The provided HTML parser.
mod parser;

/// The state management for the HTML parser.
mod state;

pub use parser::HtmlStreamParser;
pub use state::{BlockedReason, ParserState};
