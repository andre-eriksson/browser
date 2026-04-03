use browser_core::TabId;
use iced::{Task, window::Id};

use crate::{
    core::{Application, UiTab},
    events::Event,
};

/// Handles the addition of a new tab to the application when a `TabAdded` event is received from the browser.
pub(crate) fn on_new_tab(application: &mut Application, window_id: Id, new_tab_id: TabId) -> Task<Event> {
    if let Some(ctx) = application.browser_windows.get_mut(&window_id) {
        let new_tab = UiTab::new(new_tab_id);
        ctx.tabs.push(new_tab);
    }

    Task::none()
}

/// Handles the closure of a tab when a `TabClosed` event is received from the browser.
/// It removes the closed tab from the application's state and updates the active tab if necessary.
pub(crate) fn on_close_tab(
    application: &mut Application,
    window_id: Id,
    tab_id: TabId,
    next_tab_id: Option<TabId>,
) -> Task<Event> {
    if let Some(ctx) = application.browser_windows.get_mut(&window_id)
        && let Some(next_id) = next_tab_id
    {
        ctx.tabs.retain(|tab| tab.id != tab_id);
        ctx.active_tab_id = next_id;
    }

    Task::none()
}

/// Handles the switching of the active tab when an `ActiveTabChanged` event is received from the browser.
pub(crate) fn on_switch_tab(application: &mut Application, window_id: Id, tab_id: TabId) -> Task<Event> {
    if let Some(ctx) = application.browser_windows.get_mut(&window_id) {
        ctx.active_tab_id = tab_id;
    }

    Task::none()
}
