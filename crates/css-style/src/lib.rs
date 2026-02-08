//! CSS Style Module
//!
//! This crate implements the functionality to turn CSS stylesheets into
//! computed styles for HTML elements, following the CSS specifications.

mod calculate;
mod cascade;
mod computed;
mod handler;
mod primitives;
mod properties;
mod tree;

pub use computed::ComputedStyle;
pub use primitives::*;
pub use properties::border::*;
pub use properties::color::*;
pub use properties::dimension::*;
pub use properties::display::*;
pub use properties::font::*;
pub use properties::offset::*;
pub use properties::position::*;
pub use properties::text::*;
pub use properties::{AbsoluteContext, CSSProperty, RelativeContext, RelativeType};
pub use tree::{StyleTree, StyledNode};
