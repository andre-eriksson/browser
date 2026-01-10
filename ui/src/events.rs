use browser_core::TabId;
use iced::window;

use crate::api::window::WindowType;

/// Represents the different types of UI events that can occur in the application.
#[derive(Debug, Clone)]
pub enum UiEvent {
    /// Create a new window of the specified type.
    NewWindow(WindowType),

    /// Close the window with the specified ID.
    CloseWindow(window::Id),

    /// Handle window resize event with new width and height.
    WindowResized(window::Id, f32, f32),

    /// Create a new tab.
    NewTab,

    /// Close the tab with the specified ID.
    CloseTab(TabId),

    /// Change the active tab to the tab with the specified ID.
    ChangeActiveTab(TabId),

    /// Change the URL in the address bar to the specified URL.
    ChangeURL(String),

    /// Handle content scroll event with new scroll offset.
    ContentScrolled(f32, f32),
}
