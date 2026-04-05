use std::sync::Arc;

use crate::Page;

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
        if self
            .backward
            .last()
            .is_some_and(|last| Arc::ptr_eq(last, &current))
        {
            return;
        }

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

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}
