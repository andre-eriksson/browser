use std::sync::Arc;

use crate::errors::{BrowserError, NavigationError};
use async_trait::async_trait;
use network::HeaderMap;

use crate::tab::{page::Page, tabs::TabId};

#[async_trait]
pub trait Commandable {
    async fn execute(&mut self, command: BrowserCommand) -> Result<BrowserEvent, BrowserError>;
}

/// A trait representing an event emitter that can emit events of type `T`.
pub trait Emitter<T>: Send + Sync {
    fn emit(&self, event: T);
    fn clone_box(&self) -> Box<dyn Emitter<T>>;
}

/// Represents various events that can occur within the browser.
#[derive(Debug, Clone)]
pub enum BrowserEvent {
    /// A new tab has been added.
    TabAdded(TabId),

    /// A tab has been closed.
    TabClosed(TabId, Option<TabId>),

    /// The active tab has changed.
    ActiveTabChanged(TabId),

    /// Navigate to the specified URL.
    NavigateTo(String),

    /// Navigation succeeded.
    NavigateSuccess(TabId, Arc<Page>),

    /// Navigation failed with a network error.
    NavigateError(NavigationError),

    /// An image was successfully fetched from the network.
    ImageFetched(TabId, String, Vec<u8>, HeaderMap),

    /// A general browser error occurred (for errors that don't fit other categories).
    Error(BrowserError),
}

/// Represents commands that can be issued to the browser.
#[derive(Debug)]
pub enum BrowserCommand {
    /// Command to navigate a tab to a specified URL.
    Navigate { tab_id: TabId, url: String },

    /// Command to add a new tab.
    AddTab,

    /// Command to close an existing tab.
    CloseTab { tab_id: TabId },

    /// Command to change the active tab.
    ChangeActiveTab { tab_id: TabId },

    /// Command to fetch an image resource using the browser's HTTP client, headers, and cookies.
    FetchImage { tab_id: TabId, url: String },
}

impl BrowserCommand {
    pub fn parse_navigate(value: &str) -> Option<Self> {
        let parts: Vec<&str> = value.splitn(2, ' ').collect();
        if parts.len() != 2 {
            return None;
        }
        let tab_id = parts[0].parse::<usize>().ok()?;
        let url = parts[1].to_string();

        Some(BrowserCommand::Navigate {
            tab_id: TabId(tab_id),
            url,
        })
    }
}
