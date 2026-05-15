use html_dom::NodeId;
use iced::{Size, Task, window::Id};
use layout::{LayoutImage, LayoutTree};
use tracing::error;

use crate::{
    core::{Application, Tab, TabId},
    errors::BrowserError,
    events::{Event, EventHandler},
    windows::browser::window::BrowserWindow,
};

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
    ImageDecoded {
        window_id: Id,
        tab_id: TabId,
        node_ids: Vec<NodeId>,
        url: String,
        image_data: LayoutImage,
    },

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
            BrowserEvent::NewTab(window_id) => Tab::create_new_tab(self, window_id),
            BrowserEvent::CloseTab(window_id, tab_id) => Tab::close_tab(self, window_id, tab_id),
            BrowserEvent::ChangeActiveTab(window_id, tab_id) => Tab::change_active_tab(self, window_id, tab_id),

            BrowserEvent::NavigateBack(window_id) => Tab::navigate_back(self, window_id),
            BrowserEvent::NavigateForward(window_id) => Tab::navigate_forward(self, window_id),
            BrowserEvent::Refresh(window_id) => Tab::refresh_page(self, window_id),

            BrowserEvent::ChangeURL(window_id, url) => BrowserWindow::on_url_change(self, window_id, url),
            BrowserEvent::Scroll(window_id, x, y) => BrowserWindow::on_scrolled(self, window_id, x, y),
            BrowserEvent::Resize(window_id, new_viewport) => BrowserWindow::on_resized(self, window_id, new_viewport),

            BrowserEvent::ImageDecoded {
                window_id,
                tab_id,
                node_ids,
                url,
                image_data,
            } => Tab::on_image_decoded(self, window_id, tab_id, node_ids, url, image_data),

            BrowserEvent::RelayoutComplete(window_id, tab_id, generation, layout_tree) => {
                Tab::on_relayout(self, window_id, tab_id, generation, layout_tree)
            }
            BrowserEvent::Error(error) => {
                error!(%error, "Browser error occurred");
                Task::none()
            }
        }
    }
}
