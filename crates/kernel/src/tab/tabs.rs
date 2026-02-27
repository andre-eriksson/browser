use std::{fmt::Display, sync::Arc};

use io::DocumentPolicy;

use crate::tab::page::Page;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(pub usize);

impl Display for TabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: TabId,
    page: Arc<Page>,
    policies: DocumentPolicy,
}

impl Tab {
    pub fn new(id: TabId) -> Self {
        Tab {
            id,
            page: Page::blank().into(),
            policies: DocumentPolicy::default(),
        }
    }

    pub fn page(&self) -> &Arc<Page> {
        &self.page
    }

    pub fn set_page(&mut self, page: Arc<Page>) {
        self.page = page;
    }

    pub fn policies(&self) -> &DocumentPolicy {
        &self.policies
    }
}
