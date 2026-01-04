use browser_core::tab::TabId;
use iced::window;

use crate::api::window::WindowType;

/// Represents the different types of UI events that can occur in the application.
#[derive(Debug, Clone)]
pub enum UiEvent {
    /// Create a new window of the specified type.
    NewWindow(WindowType),

    /// Close the window with the specified ID.
    CloseWindow(window::Id),

    /// Create a new tab.
    NewTab,

    /// Close the tab with the specified ID.
    CloseTab(TabId),

    /// Change the active tab to the tab with the specified ID.
    ChangeActiveTab(TabId),

    /// Change the URL in the address bar to the specified URL.
    ChangeURL(String),
}
