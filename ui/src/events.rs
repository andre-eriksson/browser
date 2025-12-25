use browser_core::tab::TabId;
use iced::window;

use crate::api::window::WindowType;

#[derive(Debug, Clone)]
pub enum UiEvent {
    // Window Events
    NewWindow(WindowType),
    CloseWindow(window::Id),

    // Tab Events
    NewTab,
    CloseTab(TabId),
    ChangeActiveTab(TabId),
    ChangeURL(String),
}
