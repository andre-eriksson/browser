use crate::tab::TabId;

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
