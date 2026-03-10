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

#[derive(Debug, Clone, Default, Copy)]
pub struct HistoryState {
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

#[derive(Debug, Clone)]
pub struct History {
    forward: Vec<Arc<Page>>,
    backward: Vec<Arc<Page>>,
}

impl History {
    pub fn new() -> Self {
        History {
            forward: Vec::with_capacity(10),
            backward: Vec::with_capacity(10),
        }
    }

    pub fn push(&mut self, current: Arc<Page>) {
        if current.document_url().is_some() {
            self.backward.push(current);
        }

        self.forward.clear();
    }

    pub fn can_go_back(&self) -> bool {
        !self.backward.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.forward.is_empty()
    }

    pub fn go_back(&mut self, current: Arc<Page>) -> Option<Arc<Page>> {
        if let Some(page) = self.backward.pop() {
            self.forward.push(current);
            Some(page)
        } else {
            None
        }
    }

    pub fn go_forward(&mut self, current: Arc<Page>) -> Option<Arc<Page>> {
        if let Some(page) = self.forward.pop() {
            self.backward.push(current);
            Some(page)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: TabId,
    page: Arc<Page>,
    history: History,
    policies: DocumentPolicy,
}

impl Tab {
    pub fn new(id: TabId) -> Self {
        Tab {
            id,
            page: Page::blank().into(),
            history: History::new(),
            policies: DocumentPolicy::default(),
        }
    }

    pub fn history_state(&self) -> HistoryState {
        HistoryState {
            can_go_back: self.history.can_go_back(),
            can_go_forward: self.history.can_go_forward(),
        }
    }

    pub fn navigate_to(&mut self, page: Arc<Page>) {
        if self.page.document_url() != page.document_url() {
            self.history.push(self.page.clone());
        }
        self.page = page;
    }

    pub fn navigate_back(&mut self) -> bool {
        if let Some(page) = self.history.go_back(self.page.clone()) {
            self.page = page;
            true
        } else {
            false
        }
    }

    pub fn navigate_forward(&mut self) -> bool {
        if let Some(page) = self.history.go_forward(self.page.clone()) {
            self.page = page;
            true
        } else {
            false
        }
    }

    pub fn page(&self) -> &Arc<Page> {
        &self.page
    }

    pub fn policies(&self) -> &DocumentPolicy {
        &self.policies
    }
}
