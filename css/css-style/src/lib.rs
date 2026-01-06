//! CSS Style Module
//!
//! This crate implements the functionality to turn CSS stylesheets into
//! computed styles for HTML elements, following the CSS specifications.

/// Calculates the cascading of CSS declarations to determine final property values.
mod cascade;

/// The computed styles for HTML elements.
mod computed;

/// Handles CSS inheritance rules for properties.
mod inheritance;

/// Resolves CSS property values from strings to usable types.
mod resolver;

/// Represents the style tree structure for applying styles.
mod tree;

/// Defines various types used in CSS styling.
pub mod types;

pub use tree::StyleTree;
