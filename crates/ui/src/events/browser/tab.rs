use iced::Task;
use kernel::TabId;

use crate::{
    core::{Application, UiTab},
    events::Event,
};

/// Handles the addition of a new tab to the application when a `TabAdded` event is received from the browser.
pub(crate) fn on_new_tab(application: &mut Application, new_tab_id: TabId) -> Task<Event> {
    let new_tab = UiTab::new(new_tab_id);
    application.tabs.push(new_tab);

    Task::none()
}

/// Handles the closure of a tab when a `TabClosed` event is received from the browser.
/// It removes the closed tab from the application's state and updates the active tab if necessary.
pub(crate) fn on_close_tab(
    application: &mut Application,
    tab_id: TabId,
    next_tab_id: Option<TabId>,
) -> Task<Event> {
    application.tabs.retain(|tab| tab.id != tab_id);

    if let Some(next_id) = next_tab_id {
        application.active_tab = next_id;
    }

    Task::none()
}

/// Handles the switching of the active tab when an `ActiveTabChanged` event is received from the browser.
pub(crate) fn on_switch_tab(application: &mut Application, tab_id: TabId) -> Task<Event> {
    application.active_tab = tab_id;
    Task::none()
}
