use browser_core::{Commandable, EngineCommand};
use iced::{Task, window, window::Id};

use crate::{
    core::{Application, WindowType},
    events::Event,
    views::browser::window::BrowserContext,
};

/// Handles the creation of a new window when a `NewWindow` event is received from the UI.
pub(crate) fn create_window(application: &mut Application, window_id: Id, window_type: WindowType) -> Task<Event> {
    match window_type {
        WindowType::Devtools => {
            // TODO: Devtools should be a child of the browser window.

            let browser = application.browser.clone();

            Task::perform(
                async move {
                    let mut lock = browser.lock().await;
                    let active_tab = lock.tab_manager().active_tab_id();
                    lock.execute(EngineCommand::GetDevtoolsPage { tab_id: active_tab })
                        .await
                },
                move |result| match result {
                    Ok(event) => Event::EngineResponse(window_id, event),
                    Err(e) => {
                        panic!("Failed to get devtools page: {:?}", e);
                    }
                },
            )
        }
        WindowType::Browser => {
            let (id, task) = application.window_controller.new_window(None, window_type);

            application
                .browser_windows
                .insert(id, BrowserContext::new(application.config));

            task.discard()
        }
    }
}

/// Handles the closure of a window when a `CloseWindow` event is received from the UI.
pub(crate) fn close_window(application: &mut Application, window_id: Id) -> Task<Event> {
    let mut windows_to_close = vec![window_id];

    if let Some(ctx) = application.browser_windows.remove(&window_id) {
        windows_to_close.extend(
            ctx.tabs
                .into_iter()
                .filter_map(|tab| tab.devtools.map(|devtools| devtools.window_id)),
        );
    } else {
        for ctx in application.browser_windows.values_mut() {
            for tab in &mut ctx.tabs {
                if tab
                    .devtools
                    .as_ref()
                    .is_some_and(|devtools| devtools.window_id == window_id)
                {
                    tab.devtools = None;
                }
            }
        }
    }

    for id in &windows_to_close {
        application.window_controller.close(*id);
    }

    let mut close_tasks: Vec<Task<()>> = windows_to_close.into_iter().map(window::close).collect();

    if application.browser_windows.is_empty() {
        if application.window_controller.open_windows.is_empty() {
            return iced::exit();
        }

        close_tasks.push(application.window_controller.close_all_windows());
    }

    Task::batch(close_tasks).discard()
}
