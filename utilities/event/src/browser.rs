use errors::network::NetworkError;

/// Represents various events that can occur within the browser.
#[derive(Debug, Clone)]
pub enum BrowserEvent {
    // === Tab Management ===
    /// Open a new tab in the browser.
    OpenNewTab,

    /// Close an existing tab identified by its index.
    CloseTab(usize),

    /// Change the active tab to the one identified by its index.
    ChangeTab(usize),

    /// Change the URL of the current tab to the specified URL.
    ChangeURL(String),

    // === Navigation ===
    /// Navigate to the specified URL.
    NavigateTo(String),

    /// Navigation succeeded.
    NavigateSuccess,

    /// Navigation failed with a network error.
    NavigateError(NetworkError),

    // === UI Updates ===
    /// Refresh the content of the current tab.
    RefreshContent,
}
