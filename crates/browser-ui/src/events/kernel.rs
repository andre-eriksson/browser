use browser_core::EngineResponse;
use iced::{Task, window::Id};
use tracing::error;

use crate::{
    core::{Application, TabId},
    events::{
        Event, EventHandler,
        kernel::{
            request::navigate::navigate_to_url,
            response::{
                devtools::on_devtools_page_ready,
                navigate::{on_image_loaded, on_navigation_error, on_navigation_success},
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
}

impl EventHandler<EngineRequest> for Application {
    fn handle(&mut self, event: EngineRequest) -> Task<Event> {
        match event {
            EngineRequest::NavigateTo(window_id, url) => navigate_to_url(self, window_id, url),
        }
    }
}

impl EventHandler<(Id, TabId, Box<EngineResponse>)> for Application {
    fn handle(&mut self, event: (Id, TabId, Box<EngineResponse>)) -> Task<Event> {
        let window_id = event.0;
        let tab_id = event.1;
        let response = event.2;

        match *response {
            EngineResponse::DevtoolsPageReady(page) => on_devtools_page_ready(self, window_id, tab_id, page),

            EngineResponse::NavigateSuccess(page, metadata, navigation_type) => {
                on_navigation_success(self, window_id, tab_id, page, metadata, navigation_type)
            }
            EngineResponse::NavigateError(error) => on_navigation_error(self, error),

            EngineResponse::ImageFetched(url, bytes, headers) => {
                on_image_loaded(self, window_id, tab_id, url, bytes, headers)
            }

            EngineResponse::Error(error) => {
                error!(%error, "Engine command failed");
                Task::none()
            }
        }
    }
}
