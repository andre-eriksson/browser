//! Main browser UI, built with iced.

/// Core components of the UI module.
mod core;

/// UI event definitions.
mod events;

/// The runtime environment for the UI.
mod runtime;

/// Utility functions and types.
mod util;

/// UI views and components.
mod windows;

mod renderer;

pub mod errors;

pub use runtime::Ui;
pub use util::fonts::load_fallback_fonts;
