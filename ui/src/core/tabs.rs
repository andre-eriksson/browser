use browser_core::tab::TabId;

pub struct UiTab {
    pub id: TabId,
    pub title: Option<String>,
}

impl UiTab {
    pub fn new(id: TabId) -> Self {
        Self { id, title: None }
    }
}
