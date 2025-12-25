use crate::tab::TabId;

#[derive(Debug)]
pub enum BrowserCommand {
    // Commands related to tab management
    Navigate { tab_id: TabId, url: String },
    AddTab { url: Option<String> },
    CloseTab { tab_id: TabId },
    ChangeActiveTab { tab_id: TabId },
}
