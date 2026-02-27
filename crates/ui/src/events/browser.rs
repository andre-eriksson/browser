use iced::Task;
use kernel::BrowserEvent;

use crate::{
    core::Application,
    events::{
        Event, EventHandler,
        browser::{
            navigate::{navigate_to_url, on_image_loaded, on_navigation_error, on_navigation_success},
            tab::{on_close_tab, on_new_tab, on_switch_tab},
        },
    },
};

mod navigate;
mod tab;

impl EventHandler<BrowserEvent> for Application {
    fn handle(&mut self, event: BrowserEvent) -> Task<Event> {
        match event {
            BrowserEvent::TabAdded(new_tab_id) => on_new_tab(self, new_tab_id),
            BrowserEvent::TabClosed(tab_id, next_tab_id) => on_close_tab(self, tab_id, next_tab_id),
            BrowserEvent::ActiveTabChanged(tab_id) => on_switch_tab(self, tab_id),

            BrowserEvent::NavigateTo(new_url) => navigate_to_url(self, new_url),
            BrowserEvent::NavigateSuccess(tab_id, page) => on_navigation_success(self, tab_id, page),
            BrowserEvent::NavigateError(error) => on_navigation_error(self, error),

            BrowserEvent::ImageFetched(tab_id, url, bytes, headers) => {
                on_image_loaded(self, tab_id, url, bytes, headers)
            }

            BrowserEvent::Error(error) => {
                tracing::error!("Browser error: {:?}", error);
                Task::none()
            }
        }
    }
}
