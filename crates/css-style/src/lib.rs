//! CSS Style Module
//!
//! This crate implements the functionality to turn CSS stylesheets into
//! computed styles for HTML elements, following the CSS specifications.

/// Cached stylesheet wrapper for pre-computed selector sequences.
pub(crate) mod cached_stylesheet;

/// Calculates the cascading of CSS declarations to determine final property values.
mod cascade;

/// The computed styles for HTML elements.
mod computed;

/// Resolves CSS property values from strings to usable types.
mod resolver;

/// Represents the style tree structure for applying styles.
mod tree;

/// Defines various types used in CSS styling.
pub mod types;

pub use computed::ComputedStyle;
pub use tree::{StyleTree, StyledNode};
