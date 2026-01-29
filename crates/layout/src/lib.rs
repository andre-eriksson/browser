//! Layout engine for rendering web pages.
//!
//! This module provides the core functionality for constructing and managing
//! the layout tree, which represents the visual structure of a web page based
//! on the DOM and CSS styles.

/// Layout engine module
pub mod engine;

/// Layout tree and nodes
mod layout;

/// Primitive types for layout calculations
mod primitives;

/// Text measurement context
mod text;

mod mode;
mod resolver;

pub use engine::LayoutEngine;
pub use layout::{LayoutColors, LayoutNode, LayoutTree};
pub use primitives::{Color4f, Rect, SideOffset};
pub use text::TextContext;
