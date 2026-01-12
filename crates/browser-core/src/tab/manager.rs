use errors::browser::TabError;

use crate::tab::tabs::{Tab, TabId};

pub struct TabManager {
    active_tab: TabId,
    tabs: Vec<Tab>,
    next_tab_id: usize,
}

impl TabManager {
    pub fn new(initial_tab: Tab) -> Self {
        TabManager {
            active_tab: initial_tab.id,
            tabs: vec![initial_tab],
            next_tab_id: 1,
        }
    }

    pub(crate) fn active_tab_id(&self) -> TabId {
        self.active_tab
    }

    pub(crate) fn active_tab(&self) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == self.active_tab)
    }

    pub(crate) fn get_tab_mut(&mut self, tab_id: TabId) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|t| t.id == tab_id)
    }

    pub(crate) fn next_tab_id(&self) -> usize {
        self.next_tab_id
    }

    pub(crate) fn add_tab(&mut self, tab: Tab) {
        self.tabs.push(tab);
        self.next_tab_id += 1;
    }

    pub(crate) fn change_active_tab(&mut self, tab_id: TabId) -> Result<(), TabError> {
        if !self.tabs.iter().any(|t| t.id == tab_id) {
            return Err(TabError::TabNotFound(tab_id.0));
        }

        self.active_tab = tab_id;

        Ok(())
    }

    pub(crate) fn change_to_any_tab(&mut self) -> Result<(), TabError> {
        if let Some(first_tab) = self.tabs.first() {
            self.change_active_tab(first_tab.id)?;
            Ok(())
        } else {
            Err(TabError::NoTabsAvailable)
        }
    }

    pub(crate) fn close_tab(&mut self, tab_id: TabId) -> Result<(), TabError> {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == tab_id) {
            self.tabs.remove(pos);
            Ok(())
        } else {
            Err(TabError::TabNotFound(tab_id.0))
        }
    }
}
