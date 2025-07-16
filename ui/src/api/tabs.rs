use std::sync::{Arc, Mutex};

use api::{
    collector::{Collector, TagInfo},
    dom::{ArcDomNode, ConvertDom, DomNode},
};

/// A collector that gathers metadata from HTML tags in a browser tab.
///
/// # Fields
/// * `in_head` - A boolean indicating whether the collector is currently processing the `<head>` section of the HTML document.
/// * `url` - The URL of the page being collected.
/// * `title` - The title of the page, if available.
/// * `external_resources` - A vector of tuples containing DOM nodes and their associated resource URLs (e.g., scripts, stylesheets).
#[derive(Debug, Default)]
pub struct TabCollector {
    pub in_head: bool,
    pub url: String,
    pub title: Option<String>,
    pub favicons: Vec<(ArcDomNode, String)>,
}

impl Collector for TabCollector {
    type Output = Self;

    fn collect(&mut self, tag: &TagInfo) {
        if tag.tag_name == "head" {
            self.in_head = true;
            return;
        }

        if tag.tag_name == "body" {
            self.in_head = false;
            return;
        }

        if !self.in_head {
            return;
        }

        if tag.tag_name == "link" {
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
                    self.favicons.push((tag.dom_node.clone().convert(), href));
                }
            }
        }

        if tag.tag_name == "title" {
            if let DomNode::Text(ref text) = *tag.dom_node.borrow() {
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
/// * `url` - The URL of the page loaded in the tab.
/// * `status_code` - A mutex-protected string representing the HTTP status code of the page.
/// * `html_content` - A shared DOM node containing the parsed HTML content of the page.
/// * `metadata` - Metadata about the tab, including the title and external resources.
#[derive(Debug, Clone)]
pub struct BrowserTab {
    pub id: usize,
    pub temp_url: String,
    pub url: String,
    pub html_content: ArcDomNode,
    pub metadata: TabMetadata,
}

impl Default for BrowserTab {
    fn default() -> Self {
        let url = "about:blank".to_string();
        Self {
            id: 0,
            temp_url: url.clone(),
            url: url,
            html_content: ArcDomNode::default(),
            metadata: Arc::new(Mutex::new(TabCollector::default())),
        }
    }
}
