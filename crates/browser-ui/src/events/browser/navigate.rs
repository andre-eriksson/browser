use std::sync::Arc;

use browser_core::{Commandable, EngineCommand, EngineResponse, errors::KernelError};
use iced::{Task, window::Id};

use crate::{
    core::Application,
    errors::{BrowserError, TabError},
    events::{Event, browser::BrowserEvent},
};

/// Handles navigation back in the tab's history by sending a `NavigateBack` command to the browser and processing the result,
/// including handling any navigation errors that may occur (e.g., no history to navigate back to).
pub(crate) fn navigate_back(application: &mut Application, window_id: Id) -> Task<Event> {
    let tab = application
        .browser_windows
        .get_mut(&window_id)
        .expect("No browser context found for window ID")
        .tab_manager
        .active_tab_mut()
        .expect("There should always be an active tab in the browser");

    let page = tab.history.go_forward(Arc::clone(&tab.page)).clone();

    match page {
        Some(page) => {
            tab.page = page;
            Task::none()
        }
        None => Task::done(Event::Browser(BrowserEvent::Error(BrowserError::TabError(TabError::NoHistory)))),
    }
}

/// Handles navigation forward in the tab's history by sending a `NavigateForward` command to the browser and processing the result,
/// including handling any navigation errors that may occur (e.g., no history to navigate forward to).
pub(crate) fn navigate_forward(application: &mut Application, window_id: Id) -> Task<Event> {
    let tab = application
        .browser_windows
        .get_mut(&window_id)
        .expect("No browser context found for window ID")
        .tab_manager
        .active_tab_mut()
        .expect("There should always be an active tab in the browser");

    let page = tab.history.go_forward(Arc::clone(&tab.page)).clone();

    match page {
        Some(page) => {
            tab.page = page;
            Task::none()
        }
        None => Task::done(Event::Browser(BrowserEvent::Error(BrowserError::TabError(TabError::NoHistory)))),
    }
}

/// Handles refreshing the current page by re-navigating to the current URL. It retrieves the current URL from the active tab's page
/// information and sends a `Navigate` command to the browser with that URL. If the current URL is empty
/// (e.g., if the tab has no page loaded), it simply returns without performing any action.
pub(crate) fn refresh_page(application: &mut Application, window_id: Id) -> Task<Event> {
    let tab = application
        .browser_windows
        .get(&window_id)
        .expect("No browser context found for window ID")
        .tab_manager
        .active_tab()
        .expect("There should always be an active tab in the browser");

    let Some(url) = tab.page.document_url() else {
        return Task::none();
    };

    let tab_id = tab.id;
    let url = url.to_string();
    let browser = Arc::clone(&application.browser);

    Task::perform(
        async move {
            let mut lock = browser.lock().await;

            lock.execute(EngineCommand::Navigate { url }).await
        },
        move |result| match result {
            Ok(event) => Event::EngineResponse(window_id, tab_id, event),
            Err(err) => match err {
                KernelError::NavigationError(nav_err) => {
                    Event::EngineResponse(window_id, tab_id, EngineResponse::NavigateError(nav_err))
                }
                _ => Event::EngineResponse(window_id, tab_id, EngineResponse::Error(err)),
            },
        },
    )
}
