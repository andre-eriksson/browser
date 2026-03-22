use ::layout::LayoutTree;
use iced::Task;
use kernel::TabId;

use crate::{
    core::Application,
    events::{
        Event, EventHandler,
        browser::{
            post::{on_image_loaded, on_relayout_complete},
            tab::{change_active_tab, close_tab, create_new_tab},
            window::{on_scrolled, on_url_change},
        },
    },
};

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
    NewTab,

    /// Close the tab with the specified ID.
    CloseTab(TabId),

    /// Change the active tab to the tab with the specified ID.
    ChangeActiveTab(TabId),

    /// Change the URL in the address bar to the specified URL.
    ChangeURL(String),

    /// Handle content scroll event with new scroll offset.
    Scroll(f32, f32),

    /// An image has finished loading (or failed). The first String is the source URL,
    /// the second is the pre-resolved Vary string for exact disk cache lookups.
    ImageLoaded(TabId, String, String),

    /// A background relayout has completed.  Carries the tab id, the layout
    /// generation the work was started with, and the resulting layout tree.
    /// If the generation no longer matches the tab's current generation the
    /// result is stale (e.g. the user navigated away) and should be discarded.
    RelayoutComplete(TabId, u64, LayoutTree),
}

impl EventHandler<BrowserEvent> for Application {
    fn handle(&mut self, event: BrowserEvent) -> Task<Event> {
        match event {
            BrowserEvent::ChangeURL(url) => on_url_change(self, url),

            BrowserEvent::Scroll(x, y) => on_scrolled(self, x, y),

            BrowserEvent::NewTab => create_new_tab(self),
            BrowserEvent::CloseTab(tab_id) => close_tab(self, tab_id),
            BrowserEvent::ChangeActiveTab(tab_id) => change_active_tab(self, tab_id),

            BrowserEvent::ImageLoaded(tab_id, ref url, ref vary_key) => on_image_loaded(self, tab_id, url, vary_key),
            BrowserEvent::RelayoutComplete(tab_id, generation, layout_tree) => {
                on_relayout_complete(self, tab_id, generation, layout_tree)
            }
        }
    }
}
