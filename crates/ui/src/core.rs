//! Core components of the UI module.

/// Main application and event definitions.
mod app;

/// Tab representation and related types.
mod tabs;

/// Application window representation and types.
mod window;

pub use app::Application;
pub use tabs::{ScrollOffset, UiTab};
pub use window::{ApplicationWindow, WindowType};
