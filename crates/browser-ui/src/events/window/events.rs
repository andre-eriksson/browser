use browser_core::{Commandable, EngineCommand};
use iced::{Task, window::Id};

use crate::{
    core::{Application, WindowType},
    events::Event,
};

/// Handles the creation of a new window when a `NewWindow` event is received from the UI.
pub(crate) fn create_window(application: &mut Application, window_type: WindowType) -> Task<Event> {
    match window_type {
        WindowType::Devtools => {
            let tab_id = application.active_tab;
            let browser = application.browser.clone();

            Task::perform(
                async move {
                    let mut lock = browser.lock().await;
                    lock.execute(EngineCommand::GetDevtoolsPage { tab_id })
                        .await
                },
                |result| match result {
                    Ok(event) => Event::EngineResponse(event),
                    Err(e) => {
                        panic!("Failed to get devtools page: {:?}", e);
                    }
                },
            )
        }
        _ => {
            let (_, task) = application.window_controller.new_window(window_type);
            task.discard()
        }
    }
}

/// Handles the closure of a window when a `CloseWindow` event is received from the UI.
pub(crate) fn close_window(application: &mut Application, window_id: Id) -> Task<Event> {
    application.window_controller.close(window_id);

    if window_id == application.id {
        if application.window_controller.open_windows.is_empty() {
            return iced::exit();
        } else {
            return application.window_controller.close_all_windows().discard();
        }
    }

    if application.window_controller.open_windows.is_empty() {
        return iced::exit();
    }

    Task::none()
}
