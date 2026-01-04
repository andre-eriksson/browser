use browser_core::tab::TabId;

/// Represents a tab in the UI.
#[derive(Debug, Clone)]
pub struct UiTab {
    /// The unique identifier for the tab.
    pub id: TabId,

    /// The title of the tab, if available.
    pub title: Option<String>,
}

impl UiTab {
    pub fn new(id: TabId) -> Self {
        Self { id, title: None }
    }
}
