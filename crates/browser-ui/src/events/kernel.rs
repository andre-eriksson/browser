use browser_core::EngineResponse;
use iced::{Task, window::Id};

use crate::{
    core::Application,
    events::{
        Event, EventHandler,
        kernel::{
            request::navigate::{navigate_back, navigate_forward, navigate_to_url, refresh_page},
            response::{
                devtools::on_devtools_page_ready,
                navigate::{on_image_loaded, on_navigation_error, on_navigation_success},
                tab::{on_close_tab, on_new_tab, on_switch_tab},
            },
        },
    },
};

mod request;
mod response;

/// Represents requests that can be sent to the browser kernel from the UI.
#[derive(Debug, Clone)]
pub enum EngineRequest {
    /// Navigate to the specified URL.
    NavigateTo(Id, String),

    /// Navigate back in the history of the current tab.
    NavigateBack(Id),

    /// Navigate forward in the history of the current tab.
    NavigateForward(Id),

    /// Reload the current page in the active tab.
    Refresh(Id),
}

impl EventHandler<EngineRequest> for Application {
    fn handle(&mut self, event: EngineRequest) -> Task<Event> {
        match event {
            EngineRequest::NavigateTo(window_id, url) => navigate_to_url(self, window_id, url),
            EngineRequest::NavigateBack(window_id) => navigate_back(self, window_id),
            EngineRequest::NavigateForward(window_id) => navigate_forward(self, window_id),
            EngineRequest::Refresh(window_id) => refresh_page(self, window_id),
        }
    }
}

impl EventHandler<(Id, EngineResponse)> for Application {
    fn handle(&mut self, event: (Id, EngineResponse)) -> Task<Event> {
        let window_id = event.0;
        match event.1 {
            EngineResponse::TabAdded(new_tab_id) => on_new_tab(self, window_id, new_tab_id),
            EngineResponse::TabClosed(tab_id, next_tab_id) => on_close_tab(self, window_id, tab_id, next_tab_id),
            EngineResponse::ActiveTabChanged(tab_id) => on_switch_tab(self, window_id, tab_id),

            EngineResponse::DevtoolsPageReady(tab_id, page) => on_devtools_page_ready(self, window_id, tab_id, page),

            EngineResponse::NavigateSuccess(tab_id, page, history) => {
                on_navigation_success(self, window_id, tab_id, page, history)
            }
            EngineResponse::NavigateError(error) => on_navigation_error(self, error),

            EngineResponse::ImageFetched(tab_id, url, bytes, headers) => {
                on_image_loaded(self, window_id, tab_id, url, bytes, headers)
            }

            EngineResponse::Error(error) => {
                tracing::error!("Browser error: {:?}", error);
                Task::none()
            }
        }
    }
}
