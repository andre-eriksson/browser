use browser_core::EngineResponse;
use iced::Task;

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
    NavigateTo(String),

    /// Navigate back in the history of the current tab.
    NavigateBack,

    /// Navigate forward in the history of the current tab.
    NavigateForward,

    /// Reload the current page in the active tab.
    Refresh,
}

impl EventHandler<EngineRequest> for Application {
    fn handle(&mut self, event: EngineRequest) -> Task<Event> {
        match event {
            EngineRequest::NavigateTo(url) => navigate_to_url(self, url),
            EngineRequest::NavigateBack => navigate_back(self),
            EngineRequest::NavigateForward => navigate_forward(self),
            EngineRequest::Refresh => refresh_page(self),
        }
    }
}

impl EventHandler<EngineResponse> for Application {
    fn handle(&mut self, event: EngineResponse) -> Task<Event> {
        match event {
            EngineResponse::TabAdded(new_tab_id) => on_new_tab(self, new_tab_id),
            EngineResponse::TabClosed(tab_id, next_tab_id) => on_close_tab(self, tab_id, next_tab_id),
            EngineResponse::ActiveTabChanged(tab_id) => on_switch_tab(self, tab_id),

            EngineResponse::NavigateSuccess(tab_id, page, history) => {
                on_navigation_success(self, tab_id, page, history)
            }
            EngineResponse::NavigateError(error) => on_navigation_error(self, error),

            EngineResponse::DevtoolsPageReady(tab_id, page) => on_devtools_page_ready(self, tab_id, page),

            EngineResponse::ImageFetched(tab_id, url, bytes, headers) => {
                on_image_loaded(self, tab_id, url, bytes, headers)
            }

            EngineResponse::Error(error) => {
                tracing::error!("Browser error: {:?}", error);
                Task::none()
            }
        }
    }
}
