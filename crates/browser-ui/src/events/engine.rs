use browser_core::EngineResponse;
use iced::{Task, window::Id};
use tracing::error;

use crate::{
    core::{Application, Tab, TabId},
    events::{Event, EventHandler},
    windows::devtools::window::DevtoolsWindow,
};

/// Represents requests that can be sent to the browser kernel from the UI.
#[derive(Debug, Clone)]
pub enum EngineRequest {
    /// Navigate to the specified URL.
    NavigateTo(Id, String),
}

impl EventHandler<EngineRequest> for Application {
    fn handle(&mut self, event: EngineRequest) -> Task<Event> {
        match event {
            EngineRequest::NavigateTo(window_id, url) => Tab::navigate_to_url(self, window_id, url),
        }
    }
}

impl EventHandler<(Id, TabId, Box<EngineResponse>)> for Application {
    fn handle(&mut self, event: (Id, TabId, Box<EngineResponse>)) -> Task<Event> {
        let window_id = event.0;
        let tab_id = event.1;
        let response = event.2;

        match *response {
            EngineResponse::DevtoolsPageReady(page) => DevtoolsWindow::on_ready(self, window_id, tab_id, page),

            EngineResponse::NavigateSuccess(page, metadata, navigation_type) => {
                Tab::on_navigation_success(self, window_id, tab_id, page, metadata, navigation_type)
            }
            EngineResponse::NavigateError(error) => {
                error!(%error, "Navigation failed");
                Task::none()
            }

            EngineResponse::ImageFetched {
                node_ids,
                content_type,
                url,
                data,
            } => Tab::on_image_loaded(self, window_id, tab_id, node_ids, content_type, url, data),

            EngineResponse::Error(error) => {
                error!(%error, "Engine command failed");
                Task::none()
            }
        }
    }
}
