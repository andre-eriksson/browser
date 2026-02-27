use iced::Task;

use crate::{
    core::{Application, Event},
    events::{
        UiEvent,
        ui::{
            layout::{on_image_loaded, on_relayout_complete},
            tab::{change_active_tab, close_tab, create_new_tab},
            window::{
                close_window, create_new_window, on_content_scrolled, on_url_change,
                on_window_resized,
            },
        },
    },
};

mod layout;
mod tab;
mod window;

pub struct UiEventHandler;

impl UiEventHandler {
    pub fn handle_event(application: &mut Application, event: UiEvent) -> Task<Event> {
        match event {
            UiEvent::NewWindow(window_type) => create_new_window(application, window_type),
            UiEvent::CloseWindow(window_id) => close_window(application, window_id),
            UiEvent::WindowResized(window_id, width, height) => {
                on_window_resized(application, window_id, width, height)
            }
            UiEvent::ChangeURL(url) => on_url_change(application, url),
            UiEvent::ContentScrolled(x, y) => on_content_scrolled(application, x, y),

            UiEvent::NewTab => create_new_tab(application),
            UiEvent::CloseTab(tab_id) => close_tab(application, tab_id),
            UiEvent::ChangeActiveTab(tab_id) => change_active_tab(application, tab_id),

            UiEvent::ImageLoaded(tab_id, ref url, ref vary_key) => {
                on_image_loaded(application, tab_id, url, vary_key)
            }
            UiEvent::RelayoutComplete(tab_id, generation, layout_tree) => {
                on_relayout_complete(application, tab_id, generation, layout_tree)
            }
        }
    }
}
