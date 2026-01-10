use std::fmt::Display;

use css_cssom::CSSStyleSheet;
use html_dom::{Collector, DocumentRoot, HtmlTag, KnownTag, TagInfo};

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

    pub(crate) fn tabs(&self) -> &Vec<Tab> {
        &self.tabs
    }

    pub(crate) fn tabs_mut(&mut self) -> &mut Vec<Tab> {
        &mut self.tabs
    }

    pub(crate) fn active_tab_id(&self) -> TabId {
        self.active_tab
    }

    pub(crate) fn active_tab(&self) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == self.active_tab)
    }

    pub(crate) fn next_tab_id(&self) -> usize {
        self.next_tab_id
    }

    pub(crate) fn add_tab(&mut self, tab: Tab) {
        self.tabs.push(tab);
        self.next_tab_id += 1;
    }

    pub(crate) fn change_active_tab(&mut self, tab_id: TabId) -> Result<(), String> {
        if !self.tabs.iter().any(|t| t.id == tab_id) {
            return Err(format!("Tab with ID {:?} does not exist", tab_id));
        }

        self.active_tab = tab_id;

        Ok(())
    }

    pub(crate) fn change_to_any_tab(&mut self) -> Result<(), String> {
        if let Some(first_tab) = self.tabs.first() {
            self.change_active_tab(first_tab.id)?;
            Ok(())
        } else {
            Err("No tabs available to switch to".to_string())
        }
    }

    pub(crate) fn close_tab(&mut self, tab_id: TabId) -> Result<(), String> {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == tab_id) {
            self.tabs.remove(pos);
            Ok(())
        } else {
            Err(format!("Tab with ID {:?} does not exist", tab_id))
        }
    }
}

#[derive(Default)]
pub struct TabCollector {
    /// Indicates whether the parser is currently within the `<head>` section of the HTML document.
    pub in_head: bool,

    /// Indicates whether the parser is currently within a `<title>` tag.
    pub in_title: bool,

    /// The title of the tab, if available.
    pub title: Option<String>,
}

impl Collector for TabCollector {
    fn collect(&mut self, tag: &TagInfo) {
        if *tag.tag == HtmlTag::Known(KnownTag::Head) {
            self.in_head = true;
            return;
        }

        if *tag.tag == HtmlTag::Known(KnownTag::Body) {
            self.in_head = false;
            return;
        }

        if !self.in_head {
            return;
        }

        if *tag.tag == HtmlTag::Known(KnownTag::Title) {
            self.in_title = true;
        }

        if self.in_title && tag.data.is_some() {
            self.title = Some(tag.data.unwrap().clone());
            self.in_title = false;
        }
    }

    fn into_result(self) -> Self {
        Self {
            in_head: self.in_head,
            in_title: self.in_title,
            title: self.title,
        }
    }
}

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
    document: DocumentRoot,
    stylesheets: Vec<CSSStyleSheet>,
}

impl Tab {
    pub fn new(id: TabId) -> Self {
        Tab {
            id,
            document: DocumentRoot::new(),
            stylesheets: Vec::new(),
        }
    }

    pub fn document(&self) -> &DocumentRoot {
        &self.document
    }

    pub fn set_document(&mut self, document: DocumentRoot) {
        self.document = document;
    }

    pub fn add_stylesheet(&mut self, stylesheet: CSSStyleSheet) {
        self.stylesheets.push(stylesheet);
    }

    pub fn clear_stylesheets(&mut self) {
        self.stylesheets.clear();
    }

    pub fn stylesheets(&self) -> &Vec<CSSStyleSheet> {
        &self.stylesheets
    }
}

/// Metadata that is sent to the UI when a tab is updated.
#[derive(Debug, Clone)]
pub struct TabMetadata {
    pub id: TabId,
    pub title: String,
    pub document: DocumentRoot,
    pub stylesheets: Vec<CSSStyleSheet>,
}
