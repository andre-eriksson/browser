use iced::Task;
use kernel::KernelResponse;

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
pub enum KernelRequest {
    /// Navigate to the specified URL.
    NavigateTo(String),

    /// Navigate back in the history of the current tab.
    NavigateBack,

    /// Navigate forward in the history of the current tab.
    NavigateForward,

    /// Reload the current page in the active tab.
    Refresh,
}

impl EventHandler<KernelRequest> for Application {
    fn handle(&mut self, event: KernelRequest) -> Task<Event> {
        match event {
            KernelRequest::NavigateTo(url) => navigate_to_url(self, url),
            KernelRequest::NavigateBack => navigate_back(self),
            KernelRequest::NavigateForward => navigate_forward(self),
            KernelRequest::Refresh => refresh_page(self),
        }
    }
}

impl EventHandler<KernelResponse> for Application {
    fn handle(&mut self, event: KernelResponse) -> Task<Event> {
        match event {
            KernelResponse::TabAdded(new_tab_id) => on_new_tab(self, new_tab_id),
            KernelResponse::TabClosed(tab_id, next_tab_id) => on_close_tab(self, tab_id, next_tab_id),
            KernelResponse::ActiveTabChanged(tab_id) => on_switch_tab(self, tab_id),

            KernelResponse::NavigateSuccess(tab_id, page, history) => {
                on_navigation_success(self, tab_id, page, history)
            }
            KernelResponse::NavigateError(error) => on_navigation_error(self, error),

            KernelResponse::DevtoolsPageReady(tab_id, page) => on_devtools_page_ready(self, tab_id, page),

            KernelResponse::ImageFetched(tab_id, url, bytes, headers) => {
                on_image_loaded(self, tab_id, url, bytes, headers)
            }

            KernelResponse::Error(error) => {
                tracing::error!("Browser error: {:?}", error);
                Task::none()
            }
        }
    }
}
