use async_trait::async_trait;
use errors::network::NetworkError;

use crate::tab::{TabId, TabMetadata};

#[async_trait]
pub trait Commandable {
    async fn execute(&mut self, command: BrowserCommand) -> Result<BrowserEvent, String>;
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
    TabClosed(TabId),

    /// The active tab has changed.
    ActiveTabChanged(TabId),

    /// Navigate to the specified URL.
    NavigateTo(String),

    /// Navigation succeeded.
    NavigateSuccess(TabMetadata),

    /// Navigation failed with a network error.
    NavigateError(NetworkError),
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
}
