use iced::Task;
use kernel::BrowserEvent;

use crate::{
    core::{Application, Event},
    events::browser::{
        navigate::{navigate_to_url, on_image_loaded, on_navigation_error, on_navigation_success},
        tab::{close_tab, create_new_tab, switch_tab},
    },
};

mod navigate;
mod tab;

pub struct BrowserEventHandler;

impl BrowserEventHandler {
    pub fn handle_event(application: &mut Application, event: BrowserEvent) -> Task<Event> {
        match event {
            BrowserEvent::TabAdded(new_tab_id) => create_new_tab(application, new_tab_id),
            BrowserEvent::TabClosed(tab_id, next_tab_id) => {
                close_tab(application, tab_id, next_tab_id)
            }
            BrowserEvent::ActiveTabChanged(tab_id) => switch_tab(application, tab_id),

            BrowserEvent::NavigateTo(new_url) => navigate_to_url(application, new_url),
            BrowserEvent::NavigateSuccess(tab_id, page) => {
                on_navigation_success(application, tab_id, page)
            }
            BrowserEvent::NavigateError(error) => on_navigation_error(application, error),

            BrowserEvent::ImageFetched(tab_id, url, bytes, headers) => {
                on_image_loaded(application, tab_id, url, bytes, headers)
            }
        }
    }
}
