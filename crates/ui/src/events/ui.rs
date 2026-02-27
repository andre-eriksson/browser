use iced::Task;

use crate::{
    core::Application,
    events::{
        Event, EventHandler, UiEvent,
        ui::{
            layout::{on_image_loaded, on_relayout_complete},
            tab::{change_active_tab, close_tab, create_new_tab},
            window::{close_window, create_window, on_content_scrolled, on_url_change, on_window_resized},
        },
    },
};

mod layout;
mod tab;
mod window;

impl EventHandler<UiEvent> for Application {
    fn handle(&mut self, event: UiEvent) -> Task<Event> {
        match event {
            UiEvent::NewWindow(window_type) => create_window(self, window_type),
            UiEvent::CloseWindow(window_id) => close_window(self, window_id),
            UiEvent::WindowResized(window_id, width, height) => on_window_resized(self, window_id, width, height),
            UiEvent::ChangeURL(url) => on_url_change(self, url),
            UiEvent::ContentScrolled(x, y) => on_content_scrolled(self, x, y),

            UiEvent::NewTab => create_new_tab(self),
            UiEvent::CloseTab(tab_id) => close_tab(self, tab_id),
            UiEvent::ChangeActiveTab(tab_id) => change_active_tab(self, tab_id),

            UiEvent::ImageLoaded(tab_id, ref url, ref vary_key) => on_image_loaded(self, tab_id, url, vary_key),
            UiEvent::RelayoutComplete(tab_id, generation, layout_tree) => {
                on_relayout_complete(self, tab_id, generation, layout_tree)
            }
        }
    }
}
