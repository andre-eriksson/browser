use std::sync::{Arc, Mutex};

use api::{
    collector::{Collector, TagInfo},
    dom::{ArcDomNode, ConvertDom, DomNode},
    logging::{DURATION, EVENT, EVENT_PAGE_LOADED, STATUS_CODE},
    sender::NetworkMessage,
};
use html_parser::parser::streaming::HtmlStreamParser;
use tokio::sync::{mpsc, oneshot};
use tracing::{Span, error, info};

use crate::network::{client::setup_new_client, thread::spawn_network_thread};

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
/// * `id` - A unique identifier for the tab.
/// * `url` - The URL of the page loaded in the tab.
/// * `status_code` - A mutex-protected string representing the HTTP status code of the page.
/// * `html_content` - A shared DOM node containing the parsed HTML content of the page.
/// * `metadata` - Metadata about the tab, including the title and external resources.
pub struct BrowserTab {
    pub id: usize,
    pub network_sender: mpsc::UnboundedSender<NetworkMessage>,
    pub url: String,
    pub html_content: ArcDomNode,
    pub metadata: Arc<Mutex<TabCollector>>,
    pub span: Span,
}

impl BrowserTab {
    pub fn new(id: usize, url: String) -> Self {
        let client = setup_new_client();

        if let Err(e) = client {
            error!("Failed to create new client: {}", e);
            panic!("Failed to create new client");
        }

        let client = client.unwrap();

        let span = tracing::info_span!("BrowserTab", id = id);

        let network_sender = {
            let _enter = span.enter();
            spawn_network_thread(client, span.clone())
        };

        BrowserTab {
            id,
            network_sender,
            url,
            html_content: ArcDomNode::default(),
            metadata: Arc::new(Mutex::new(TabCollector::default())),
            span,
        }
    }

    pub fn navigate_to(&mut self, url: String) {
        let _enter = self.span.enter();

        let (response_tx, response_rx) = oneshot::channel();

        self.network_sender
            .send(NetworkMessage::InitializePage {
                full_url: url.clone(),
                response: response_tx,
            })
            .expect("Failed to send InitializePage message");

        let url_clone = self.url.clone();
        let html_content_clone: ArcDomNode = self.html_content.clone();
        let metadata_clone = self.metadata.clone();
        let span_clone = self.span.clone();
        let start_time = std::time::Instant::now();

        tokio::spawn(async move {
            let _enter = span_clone.enter();

            match response_rx.await {
                Ok(network_response) => match network_response {
                    Ok(html) => {
                        let parser = HtmlStreamParser::new(html.as_bytes(), None);

                        let parsed = parser.parse(Some(TabCollector {
                            url: url_clone,
                            title: Some("Blank".to_string()),
                            external_resources: Some(Vec::new()),
                        }));

                        match parsed {
                            Ok(result) => {
                                let dom_tree = result.dom_tree;

                                //println!("Parsed DOM Tree: {:?}", dom_tree);

                                // TODO: Process the metadata, e.g. event queue, etc.
                                let mut metadata = metadata_clone.lock().unwrap();
                                *metadata = result.metadata;

                                let mut html_content = html_content_clone.lock().unwrap();
                                *html_content = dom_tree.convert().lock().unwrap().clone();
                                info!(
                                    {EVENT} = EVENT_PAGE_LOADED,
                                    {DURATION} = ?start_time.elapsed(),
                                );
                            }
                            Err(err) => {
                                error!("Parsing error: {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        if err.starts_with(STATUS_CODE) {
                            // TODO: Render an appropriate error page based on the status code.
                            //warn!("Unable to access the page: {}", err);
                        } else {
                            // TODO: Render a generic error page for website that don't exist.
                            //warn!("Failed to initialize page: {} (website doesn't exist?)", err);
                        }
                    }
                },

                Err(_) => {
                    error!("Failed to receive response from network thread.");
                }
            }
        });
    }

    /// Closes the current tab by sending a shutdown message to the network sender.
    pub fn close(&mut self) {
        let _enter = self.span.enter();

        self.network_sender
            .send(NetworkMessage::Shutdown)
            .expect("Failed to send Shutdown message");
    }
}
