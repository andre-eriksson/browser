use std::fmt::Display;

use css_cssom::CSSStyleSheet;
use html_dom::DocumentRoot;

use crate::service::network::context::NetworkContext;

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
    network_context: NetworkContext,
}

impl Tab {
    pub fn new(id: TabId) -> Self {
        Tab {
            id,
            document: DocumentRoot::new(),
            stylesheets: Vec::new(),
            network_context: NetworkContext::default(),
        }
    }

    pub fn document(&self) -> &DocumentRoot {
        &self.document
    }

    pub fn set_document(&mut self, document: DocumentRoot) {
        self.document = document;
    }

    pub fn network_context(&mut self) -> &mut NetworkContext {
        &mut self.network_context
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
