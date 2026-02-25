//! Commands module, containing various command implementations for the browser core.

mod image;
mod navigate;
mod tab;

pub(crate) use image::load_image;
pub(crate) use navigate::{navigate, resolve_request};
pub(crate) use tab::{add_tab, change_active_tab, close_tab};
