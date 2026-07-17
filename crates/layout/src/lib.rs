//! Layout engine for rendering web pages.
//!
//! This module provides the core functionality for constructing and managing
//! the layout tree, which represents the visual structure of a web page based
//! on the DOM and CSS styles.

mod context;
mod engine;
mod mode;
mod node;
mod primitives;
mod tree;

pub use context::{ImageContext, ImageData, LayoutImage, TextContext};
pub use css_style::{Color4f, Position};
pub use engine::LayoutInput;
pub(crate) use engine::LayoutState;
pub use html_dom::NodeId;
pub use node::LayoutNode;
pub use primitives::{LayoutColors, Margin, Rect};
pub use tree::LayoutTree;
