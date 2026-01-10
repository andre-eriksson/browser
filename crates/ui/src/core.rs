//! Core components of the UI module.

/// Main application and event definitions.
mod app;

/// Browser event receiver handle and stream creation.
mod handle;

/// Tab representation and related types.
mod tabs;

/// Application window representation and types.
mod window;

pub use app::{Application, Event};
pub use handle::{ReceiverHandle, create_browser_event_stream};
pub use tabs::{ScrollOffset, UiTab};
pub use window::{ApplicationWindow, WindowType};
