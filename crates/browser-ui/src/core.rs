//! Core components of the UI module.

/// Main application and event definitions.
mod app;

/// Tab representation and related types.
mod tabs;

/// Application window representation and types.
mod window;

mod handler;

pub use app::Application;
pub use tabs::{Devtools, DevtoolsContext, DevtoolsPage, Page, Tab, TabId, manager::TabManager};
pub use window::{ApplicationWindow, ScrollOffset, WindowController, WindowType};
