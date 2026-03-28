//! Commands module, containing various command implementations for the browser core.

mod html;
mod image;
mod navigate;
mod tab;

pub(crate) use html::parse_devtools_html;
pub(crate) use image::load_image;
pub(crate) use navigate::navigate;
pub(crate) use tab::{add_tab, change_active_tab, close_tab};
