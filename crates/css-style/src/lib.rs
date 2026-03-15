//! CSS Style Module
//!
//! This crate implements the functionality to turn CSS stylesheets into specified stylesheets and then into computed stylesheets.
//! It includes the logic for calculating the final computed values of CSS properties based on the specified values, the cascade,
//! and inheritance rules. The crate also defines the data structures for representing CSS properties, values, and the style tree.

mod cascade;
mod computed;
mod functions;
mod handler;
mod properties;
mod rules;
mod specified;
mod tree;

pub use computed::{
    ComputedStyle,
    color::Color4f,
    dimension::{ComputedDimension, ComputedMaxDimension},
};
pub use properties::display::*;
pub use properties::font::*;
pub use properties::offset::*;
pub use properties::position::*;
pub use properties::{AbsoluteContext, RelativeContext, RelativeType};
pub use tree::{StyleTree, StyledNode};
