//! Layout engine for rendering web pages.
//!
//! This module provides the core functionality for constructing and managing
//! the layout tree, which represents the visual structure of a web page based
//! on the DOM and CSS styles.

mod builder;
mod context;
pub mod engine;
mod float;
mod layout;
mod mode;
mod position;
mod primitives;
mod resolver;
mod text;

pub use context::ImageContext;
pub use css_style::{Color4f, Position};
pub use engine::LayoutEngine;
pub use layout::{ImageData, LayoutColors, LayoutNode, LayoutTree};
pub use primitives::{Margin, Rect};
pub use text::TextContext;
