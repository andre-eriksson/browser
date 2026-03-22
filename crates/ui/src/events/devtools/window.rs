use iced::Task;

use crate::{core::Application, events::Event};

/// Handles the scrolling of content when a `ContentScrolled` event is received from the UI,
/// updating the scroll offset of the active tab.
pub(crate) fn on_scrolled(application: &mut Application, x: f32, y: f32) -> Task<Event> {
    if let Some(devtools) = application
        .tabs
        .iter_mut()
        .find(|tab| tab.id == application.active_tab)
        .and_then(|t| t.devtools_page.as_mut())
    {
        devtools.scroll_offset.x = x;
        devtools.scroll_offset.y = y;
    }

    Task::none()
}
