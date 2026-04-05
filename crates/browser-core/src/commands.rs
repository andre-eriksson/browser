//! Commands module, containing various command implementations for the browser core.

mod html;
mod image;
mod navigate;

pub(crate) use html::parse_devtools_html;
pub(crate) use image::load_image;
pub(crate) use navigate::navigate;
