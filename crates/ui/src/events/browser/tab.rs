use iced::Task;
use kernel::TabId;

use crate::core::{Application, Event, UiTab};

pub(crate) fn create_new_tab(application: &mut Application, new_tab_id: TabId) -> Task<Event> {
    let new_tab = UiTab::new(new_tab_id);
    application.tabs.push(new_tab);

    Task::none()
}

pub(crate) fn close_tab(
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

pub(crate) fn switch_tab(application: &mut Application, tab_id: TabId) -> Task<Event> {
    application.active_tab = tab_id;
    Task::none()
}
