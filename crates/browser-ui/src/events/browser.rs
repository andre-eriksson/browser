use iced::{Size, Task, window::Id};
use layout::LayoutTree;
use tracing::error;

use crate::{
    core::{Application, TabId},
    errors::BrowserError,
    events::{
        Event, EventHandler,
        browser::{
            navigate::{navigate_back, navigate_forward, refresh_page},
            post::{on_image_loaded, on_relayout_complete},
            tab::{change_active_tab, close_tab, create_new_tab},
            window::{on_resized, on_scrolled, on_url_change},
        },
    },
};

mod navigate;
mod post;
mod tab;
mod window;

/// Represents the different types of Browser-related events that can occur in the application.
///
/// These events are specific to browser interactions and are handled by the main application logic when they are
/// triggered from the browser UI or internal browser processes.
#[derive(Debug, Clone)]
pub enum BrowserEvent {
    /// Create a new tab.
    NewTab(Id),

    /// Close the tab with the specified ID.
    CloseTab(Id, TabId),

    /// Change the active tab to the tab with the specified ID.
    ChangeActiveTab(Id, TabId),

    /// Navigate back in the history of the current tab.
    NavigateBack(Id),

    /// Navigate forward in the history of the current tab.
    NavigateForward(Id),

    /// Reload the current page in the active tab.
    Refresh(Id),

    /// Change the URL in the address bar to the specified URL.
    ChangeURL(Id, String),

    /// Handle content scroll event with new scroll offset.
    Scroll(Id, f32, f32),

    /// Handle browser resize event with new width and height.
    Resize(Id, Size),

    /// An image has finished loading (or failed). The first String is the source URL,
    /// the second is the pre-resolved Vary string for exact disk cache lookups.
    ImageLoaded(Id, TabId, String, String),

    /// A background relayout has completed.  Carries the tab id, the layout
    /// generation the work was started with, and the resulting layout tree.
    /// If the generation no longer matches the tab's current generation the
    /// result is stale (e.g. the user navigated away) and should be discarded.
    RelayoutComplete(Id, TabId, u64, LayoutTree),

    /// An error occurred during a browser operation, with the provided error message.
    Error(BrowserError),
}

impl EventHandler<BrowserEvent> for Application {
    fn handle(&mut self, event: BrowserEvent) -> Task<Event> {
        match event {
            BrowserEvent::NewTab(window_id) => create_new_tab(self, window_id),
            BrowserEvent::CloseTab(window_id, tab_id) => close_tab(self, window_id, tab_id),
            BrowserEvent::ChangeActiveTab(window_id, tab_id) => change_active_tab(self, window_id, tab_id),
            BrowserEvent::NavigateBack(window_id) => navigate_back(self, window_id),
            BrowserEvent::NavigateForward(window_id) => navigate_forward(self, window_id),
            BrowserEvent::Refresh(window_id) => refresh_page(self, window_id),

            BrowserEvent::ChangeURL(window_id, url) => on_url_change(self, window_id, url),

            BrowserEvent::Scroll(window_id, x, y) => on_scrolled(self, window_id, x, y),
            BrowserEvent::Resize(window_id, new_viewport) => on_resized(self, window_id, new_viewport),

            BrowserEvent::ImageLoaded(window_id, tab_id, ref url, ref vary_key) => {
                on_image_loaded(self, window_id, tab_id, url, vary_key)
            }
            BrowserEvent::RelayoutComplete(window_id, tab_id, generation, layout_tree) => {
                on_relayout_complete(self, window_id, tab_id, generation, layout_tree)
            }

            BrowserEvent::Error(error) => {
                error!(%error, "Browser error occurred");
                Task::none()
            }
        }
    }
}
