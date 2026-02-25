//! Layout engine for rendering web pages.
//!
//! This module provides the core functionality for constructing and managing
//! the layout tree, which represents the visual structure of a web page based
//! on the DOM and CSS styles.

/// Image dimension context for relayout
mod context;

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

pub use context::ImageContext;
pub use css_style::Color4f;
pub use engine::LayoutEngine;
pub use layout::{LayoutColors, LayoutNode, LayoutTree};
pub use primitives::{Rect, SideOffset};
pub use text::TextContext;
