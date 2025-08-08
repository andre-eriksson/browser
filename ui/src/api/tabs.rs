use std::sync::{Arc, Mutex};

use api::html::{HtmlTag, KnownTag};
use html_parser::{
    collector::{Collector, TagInfo},
    dom::{DocumentNode, DocumentRoot, MultiThreaded},
};

/// A collector that gathers metadata from HTML tags in a browser tab.
///
/// # Fields
/// * `in_head` - A boolean indicating whether the collector is currently processing the `<head>` section of the HTML document.
/// * `url` - The URL of the page being collected.
/// * `title` - The title of the page, if available.
/// * `external_resources` - A vector of tuples containing DOM nodes and their associated resource URLs (e.g., scripts, stylesheets).
#[derive(Default)]
pub struct TabCollector {
    pub in_head: bool,
    pub url: String,
    pub title: Option<String>,
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
///
/// # Fields
/// * `id` - A unique identifier for the tab.
/// * `temp_url` - A temporary URL used when inputing a new URL in to the address bar.
/// * `url` - The URL of the page loaded in the tab.
/// * `html_content` - A shared DOM node containing the parsed HTML content of the page.
/// * `metadata` - Metadata about the tab, including the title and external resources.
pub struct BrowserTab {
    pub id: usize,
    pub temp_url: String,
    pub url: String,
    pub html_content: DocumentRoot<MultiThreaded>,
    pub metadata: TabMetadata,
}

impl BrowserTab {
    /// Creates a new `BrowserTab` with the specified ID and URL.
    ///
    /// # Arguments
    /// * `id` - The unique identifier for the tab.
    /// * `url` - The URL of the page to be loaded in the tab.
    pub fn new(id: usize, url: String) -> Self {
        Self {
            id,
            temp_url: url.clone(),
            url,
            html_content: DocumentRoot::default(),
            metadata: Arc::new(Mutex::new(TabCollector::default())),
        }
    }
}
