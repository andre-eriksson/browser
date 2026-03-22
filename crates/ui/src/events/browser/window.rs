use iced::Task;

use crate::{core::Application, events::Event};

/// Handles the change of the current URL when a `UrlChanged` event is received from the UI.
pub(crate) fn on_url_change(application: &mut Application, url: String) -> Task<Event> {
    application.current_url = url;
    Task::none()
}

/// Handles the scrolling of content when a `ContentScrolled` event is received from the UI,
/// updating the scroll offset of the active tab.
pub(crate) fn on_scrolled(application: &mut Application, x: f32, y: f32) -> Task<Event> {
    if let Some(tab) = application
        .tabs
        .iter_mut()
        .find(|tab| tab.id == application.active_tab)
    {
        tab.scroll_offset.x = x;
        tab.scroll_offset.y = y;
    }

    Task::none()
}
