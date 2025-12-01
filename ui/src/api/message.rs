use iced::window;
use network::session::network::NetworkSession;

use crate::api::window::WindowType;

#[derive(Debug, Clone)]
pub enum Message {
    None,

    // === Window Management ===
    NewWindow(WindowType),
    CloseWindow(window::Id),

    // === Tab Management ===
    OpenNewTab,
    CloseTab(usize),
    ChangeTab(usize),
    ChangeURL(String),

    // === Navigation ===
    NavigateTo(String),
    NavigateSuccess(String, NetworkSession),
    NavigateError(String),

    // === UI Updates ===
    RefreshContent,
}
