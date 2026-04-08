use iced::{Task, window::Id};
use tracing::debug;

use crate::{
    core::{Application, TabId, UiTab},
    events::Event,
    views::browser::window::DEFAULT_URL,
};

/// Handles the creation of a new tab when a `NewTab` event is received from the UI.
pub(crate) fn create_new_tab(application: &mut Application, window_id: Id) -> Task<Event> {
    match application.browser_windows.get_mut(&window_id) {
        Some(window) => {
            let new_tab_id = window.tab_manager.next_tab_id();
            window
                .tab_manager
                .add_tab(UiTab::new(TabId::new(new_tab_id)));
        }
        None => {
            tracing::error!("No browser context found for window ID: {:?}", window_id);
        }
    };

    Task::none()
}

/// Handles the closure of a tab when a `CloseTab` event is received from the UI.
pub(crate) fn close_tab(application: &mut Application, window_id: Id, tab_id: TabId) -> Task<Event> {
    match application.browser_windows.get_mut(&window_id) {
        Some(window) => {
            if window.tab_manager.tabs().len() == 1 {
                debug!(
                    "Attempted to close the last remaining tab ID: {:?} in window ID: {:?}. Closing the last tab is not allowed.",
                    tab_id, window_id
                );
                return Task::none();
            }

            if window.tab_manager.close_tab(tab_id).is_err() {
                debug!("Attempted to close non-existent tab ID: {:?} in window ID: {:?}", tab_id, window_id);
            }

            if window.tab_manager.active_tab_id() == tab_id && window.tab_manager.change_to_any_tab().is_err() {
                debug!("No tabs left to switch to after closing tab ID: {:?} in window ID: {:?}", tab_id, window_id);
            }
        }
        None => {
            tracing::error!("No browser context found for window ID: {:?}", window_id);
        }
    };

    Task::none()
}

/// Handles the switching of the active tab when a `ChangeActiveTab` event is received from the UI.
pub(crate) fn change_active_tab(application: &mut Application, window_id: Id, tab_id: TabId) -> Task<Event> {
    match application.browser_windows.get_mut(&window_id) {
        Some(window) => {
            if window.tab_manager.change_active_tab(tab_id).is_err() {
                debug!("Attempted to change to non-existent tab ID: {:?} in window ID: {:?}", tab_id, window_id);
            }

            if let Some(url) = window
                .tab_manager
                .active_tab()
                .and_then(|tab| tab.page_ctx.as_ref().map(|ctx| &ctx.metadata.url))
            {
                window.current_url = url.to_string();
            } else {
                window.current_url = DEFAULT_URL.to_string();
            }
        }
        None => {
            tracing::error!("No browser context found for window ID: {:?}", window_id);
        }
    };

    Task::none()
}
