use browser_core::{EngineCommand, Commandable, EngineResponse, TabId};
use iced::Task;

use crate::{core::Application, events::Event};

/// Handles the creation of a new tab when a `NewTab` event is received from the UI.
pub(crate) fn create_new_tab(application: &mut Application) -> Task<Event> {
    let browser = application.browser.clone();

    Task::perform(
        async move {
            let mut lock = browser.lock().await;
            lock.execute(EngineCommand::AddTab).await
        },
        |result| match result {
            Ok(event) => Event::EngineResponse(event),
            Err(err) => Event::EngineResponse(EngineResponse::Error(err)),
        },
    )
}

/// Handles the closure of a tab when a `CloseTab` event is received from the UI.
pub(crate) fn close_tab(application: &mut Application, tab_id: TabId) -> Task<Event> {
    let browser = application.browser.clone();

    Task::perform(
        async move {
            let mut lock = browser.lock().await;
            lock.execute(EngineCommand::CloseTab { tab_id }).await
        },
        |result| match result {
            Ok(event) => Event::EngineResponse(event),
            Err(err) => Event::EngineResponse(EngineResponse::Error(err)),
        },
    )
}

/// Handles the switching of the active tab when a `ChangeActiveTab` event is received from the UI.
pub(crate) fn change_active_tab(application: &mut Application, tab_id: TabId) -> Task<Event> {
    let browser = application.browser.clone();

    Task::perform(
        async move {
            let mut lock = browser.lock().await;
            lock.execute(EngineCommand::ChangeActiveTab { tab_id }).await
        },
        |result| match result {
            Ok(event) => Event::EngineResponse(event),
            Err(err) => Event::EngineResponse(EngineResponse::Error(err)),
        },
    )
}
