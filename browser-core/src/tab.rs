use std::fmt::Display;

use html_syntax::{
    collector::{Collector, TagInfo},
    dom::DocumentRoot,
    tag::{HtmlTag, KnownTag},
};
use url::Url;

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

pub struct Tab {
    pub id: TabId,
    pub current_url: Option<Url>,
    pub document: Option<DocumentRoot>,
}

#[derive(Debug, Clone)]
pub struct TabMetadata {
    pub tab_id: TabId,
    pub title: String,
}

impl Tab {
    pub fn new(id: TabId, url: Option<Url>) -> Self {
        Tab {
            id,
            current_url: url,
            document: None,
        }
    }
}
