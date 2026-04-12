//! Commands module, containing various command implementations for the browser core.

mod html;
mod image;
mod navigate;

pub use html::parse_devtools_html;
pub use image::load_image;
pub use navigate::navigate;
