use std::fmt::Display;

use css_cssom::CSSStyleSheet;
use html_dom::DocumentRoot;
use html_syntax::{
    collector::{Collector, TagInfo},
    tag::{HtmlTag, KnownTag},
};

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
    stylesheets: Vec<CSSStyleSheet>,
}

impl Tab {
    pub fn new(id: TabId) -> Self {
        Tab {
            id,
            stylesheets: Vec::new(),
        }
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
