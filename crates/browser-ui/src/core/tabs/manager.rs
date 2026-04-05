use crate::{
    core::{UiTab, tabs::TabId},
    errors::TabError,
};

#[derive(Debug, Clone)]
pub struct TabManager {
    active_tab: TabId,
    tabs: Vec<UiTab>,
    next_tab_id: usize,
}

impl TabManager {
    pub fn new() -> Self {
        let initial_tab = UiTab::new(TabId(0));

        TabManager {
            active_tab: initial_tab.id,
            tabs: vec![initial_tab],
            next_tab_id: 1,
        }
    }

    pub fn active_tab_id(&self) -> TabId {
        self.active_tab
    }

    pub fn active_tab(&self) -> Option<&UiTab> {
        self.tabs.iter().find(|t| t.id == self.active_tab)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut UiTab> {
        self.tabs.iter_mut().find(|t| t.id == self.active_tab)
    }

    pub(crate) fn _get_tab(&self, tab_id: TabId) -> Option<&UiTab> {
        self.tabs.iter().find(|t| t.id == tab_id)
    }

    pub(crate) fn get_tab_mut(&mut self, tab_id: TabId) -> Option<&mut UiTab> {
        self.tabs.iter_mut().find(|t| t.id == tab_id)
    }

    pub(crate) fn next_tab_id(&self) -> usize {
        self.next_tab_id
    }

    pub(crate) fn add_tab(&mut self, tab: UiTab) {
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

    pub(crate) fn change_to_any_tab(&mut self) -> Result<TabId, TabError> {
        if let Some(first_tab) = self.tabs.last() {
            self.active_tab = first_tab.id;
            Ok(first_tab.id)
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

    pub(crate) fn tabs(&self) -> &[UiTab] {
        &self.tabs
    }

    pub(crate) fn tabs_mut(&mut self) -> &mut [UiTab] {
        &mut self.tabs
    }
}
