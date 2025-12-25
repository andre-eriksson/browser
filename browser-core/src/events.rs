use errors::network::NetworkError;

use crate::tab::{TabId, TabMetadata};

/// A trait representing an event emitter that can emit events of type `T`.
pub trait Emitter<T>: Send + Sync {
    fn emit(&self, event: T);
    fn clone_box(&self) -> Box<dyn Emitter<T>>;
}

/// Represents various events that can occur within the browser.
#[derive(Debug, Clone)]
pub enum BrowserEvent {
    TabAdded(TabId),
    TabClosed(TabId),
    ActiveTabChanged(TabId),

    // === Navigation ===
    /// Navigate to the specified URL.
    NavigateTo(String),

    /// Navigation succeeded.
    NavigateSuccess(TabMetadata),

    /// Navigation failed with a network error.
    NavigateError(NetworkError),
}
