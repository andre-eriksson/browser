use std::sync::{Arc, Mutex};

use html_parser::{
    collector::{Collector, TagInfo},
    dom::{DocumentNode, DocumentRoot, MultiThreaded},
};
use html_syntax::{HtmlTag, KnownTag};
use network::session::network::NetworkSession;
use url::Url;

/// A collector that gathers metadata from HTML tags in a browser tab.
#[derive(Default)]
pub struct TabCollector {
    /// Indicates whether the parser is currently within the `<head>` section of the HTML document.
    pub in_head: bool,

    /// The URL of the tab being collected.
    pub url: Option<Url>,

    /// The title of the tab, if available.
    pub title: Option<String>,

    /// A list of favicons found in the tab, each represented as a tuple of the DOM node and its URL.
    pub favicons: Vec<(DocumentNode<MultiThreaded>, String)>,
}

impl Collector for TabCollector {
    type Output = Self;

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

        if *tag.tag == HtmlTag::Known(KnownTag::Link) {
            if let Some(rel) = tag.attributes.get("rel") {
                if rel != "icon" && rel != "shortcut icon" {
                    return;
                }

                let type_attr = tag.attributes.get("type").cloned().unwrap_or_default();

                if type_attr == "image/svg+xml" {
                    // TODO: Handle SVG favicons, for now skip them.
                    return;
                }

                if let Some(href) = tag.attributes.get("href") {
                    let href = href.to_string();
                    self.favicons
                        .push((DocumentNode::from(tag.dom_node.clone()), href));
                }
            }
        }

        if *tag.tag == HtmlTag::Known(KnownTag::Title) {
            if let DocumentNode::Text(text) = tag.dom_node {
                self.title = Some(text.clone());
            }
        }
    }

    fn into_result(self) -> Self::Output {
        Self {
            in_head: self.in_head,
            url: self.url,
            title: self.title,
            favicons: self.favicons,
        }
    }
}

pub type TabMetadata = Arc<Mutex<TabCollector>>;

/// Represents a browser tab with its URL, status code, HTML content, and metadata.
pub struct BrowserTab {
    pub id: usize,
    pub temp_url: String,
    pub network_session: Option<NetworkSession>,
    pub html_content: DocumentRoot<MultiThreaded>,
    pub metadata: Option<TabMetadata>,
}

impl BrowserTab {
    pub fn empty(id: usize) -> Self {
        BrowserTab {
            id,
            temp_url: String::new(),
            network_session: None,
            html_content: DocumentRoot::default(),
            metadata: None,
        }
    }
}
