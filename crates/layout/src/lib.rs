//! Layout engine for rendering web pages.
//!
//! This module provides the core functionality for constructing and managing
//! the layout tree, which represents the visual structure of a web page based
//! on the DOM and CSS styles.

mod builder;
mod context;
pub mod engine;
mod layout;
mod mode;
mod primitives;
mod resolver;
mod text;

pub use context::ImageContext;
pub use css_style::Color4f;
pub use engine::LayoutEngine;
pub use layout::{ImageData, LayoutColors, LayoutNode, LayoutTree};
pub use primitives::{Rect, SideOffset};
pub use text::TextContext;
