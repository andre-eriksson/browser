use std::sync::Arc;

use browser_core::{Commandable, EngineCommand};
use iced::{Task, window, window::Id};
use tracing::debug;

use crate::{
    core::{Application, WindowType},
    events::Event,
    windows::browser::window::BrowserContext,
};

impl Application {
    /// Handles the creation of a new window when a `NewWindow` event is received from the UI.
    pub fn create_window(&mut self, window_id: Id, window_type: WindowType) -> Task<Event> {
        match window_type {
            WindowType::Devtools => {
                let tab = self
                    .browser_windows
                    .get_mut(&window_id)
                    .expect("No browser context found for window ID")
                    .tab_manager
                    .active_tab_mut()
                    .expect("There should always be an active tab in the browser");

                let tab_id = tab.id;
                let browser = Arc::clone(&self.browser);

                let Some(page) = &tab.page else {
                    debug!("Failed to find page");
                    return Task::none();
                };

                let document = page.document.dom().clone();
                let title = page.metadata.title.clone();

                Task::perform(
                    async move {
                        browser
                            .execute(EngineCommand::GetDevtoolsPage { title, document })
                            .await
                    },
                    move |result| match result {
                        Ok(event) => Event::EngineResponse(window_id, tab_id, Box::new(event)),
                        Err(e) => {
                            panic!("Failed to get devtools page: {e:?}");
                        }
                    },
                )
            }
            WindowType::Browser => {
                let (id, task) = self.window_controller.new_window(None, window_type);

                self.browser_windows.insert(id, BrowserContext::new(None));

                task.discard()
            }
        }
    }

    /// Handles the closure of a window when a `CloseWindow` event is received from the UI.
    pub fn close_window(&mut self, window_id: Id) -> Task<Event> {
        let mut windows_to_close = vec![window_id];

        if let Some(ctx) = self.browser_windows.remove(&window_id) {
            windows_to_close.extend(
                ctx.tab_manager
                    .tabs()
                    .iter()
                    .filter_map(|tab| tab.devtools.as_ref().map(|devtools| devtools.window_id)),
            );
        } else {
            for ctx in self.browser_windows.values_mut() {
                for tab in &mut ctx.tab_manager.tabs_mut().iter_mut() {
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
            self.window_controller.close(*id);
        }

        let mut close_tasks: Vec<Task<()>> = windows_to_close.into_iter().map(window::close).collect();

        if self.browser_windows.is_empty() {
            if self.window_controller.open_windows.is_empty() {
                return iced::exit();
            }

            close_tasks.push(self.window_controller.close_all_windows());
        }

        Task::batch(close_tasks).discard()
    }
}
