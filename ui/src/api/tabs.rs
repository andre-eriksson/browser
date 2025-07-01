use std::sync::{Arc, Mutex};

use api::{
    collector::{Collector, TagInfo},
    dom::{ArcDomNode, ConvertDom, DomNode},
};

/// A collector that gathers metadata from HTML tags in a browser tab.
///
/// # Fields
/// * `url` - The URL of the page being collected.
/// * `title` - The title of the page, if available.
/// * `external_resources` - A vector of tuples containing DOM nodes and their associated resource URLs (e.g., scripts, stylesheets).
#[derive(Default)]
pub struct TabCollector {
    pub url: String,
    pub title: Option<String>,
    pub external_resources: Option<Vec<(ArcDomNode, Arc<str>)>>,
}

impl Collector for TabCollector {
    type Output = Self;

    fn collect(&mut self, tag: &TagInfo) {
        if let Some(external_resources) = &mut self.external_resources {
            if let Some(href) = tag.attributes.get("href") {
                if tag.tag_name == "a" {
                    return; // Skip anchor tags for href collection
                }

                external_resources.push((tag.dom_node.clone().convert(), Arc::from(href.as_str())));
            }

            if let Some(src) = tag.attributes.get("src") {
                let resolved_src = if src.starts_with("http://") || src.starts_with("https://") {
                    // Absolute URL
                    src.clone()
                } else if src.starts_with("//") {
                    // Protocol-relative URL
                    if let Ok(parsed_url) = url::Url::parse(&self.url) {
                        format!("{}:{}", parsed_url.scheme(), src)
                    } else {
                        format!("http:{}", src) // Fallback to http
                    }
                } else if src.starts_with('/') {
                    // Absolute path
                    if let Ok(parsed_url) = url::Url::parse(&self.url) {
                        let mut base = format!(
                            "{}://{}",
                            parsed_url.scheme(),
                            parsed_url.host_str().unwrap_or("")
                        );
                        if let Some(port) = parsed_url.port() {
                            base.push_str(&format!(":{}", port));
                        }
                        format!("{}{}", base, src)
                    } else {
                        format!("{}{}", self.url, src)
                    }
                } else {
                    // Relative path
                    if let Ok(parsed_url) = url::Url::parse(&self.url) {
                        if let Ok(resolved) = parsed_url.join(src) {
                            resolved.to_string()
                        } else {
                            format!("{}/{}", self.url.trim_end_matches('/'), src)
                        }
                    } else {
                        format!("{}/{}", self.url.trim_end_matches('/'), src)
                    }
                };

                external_resources.push((tag.dom_node.clone().convert(), Arc::from(resolved_src)));
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
            url: self.url,
            title: self.title,
            external_resources: self.external_resources,
        }
    }
}

pub type TabMetadata = Arc<Mutex<TabCollector>>;

/// Represents a browser tab with its URL, status code, HTML content, and metadata.
///
/// # Fields
/// * `url` - The URL of the page loaded in the tab.
/// * `status_code` - A mutex-protected string representing the HTTP status code of the page.
/// * `html_content` - A shared DOM node containing the parsed HTML content of the page.
/// * `metadata` - Metadata about the tab, including the title and external resources.
pub struct BrowserTab {
    pub url: String,
    pub status_code: Arc<Mutex<String>>,
    pub html_content: ArcDomNode,
    pub metadata: Arc<Mutex<TabCollector>>,
}
