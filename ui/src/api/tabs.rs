use std::sync::{Arc, Mutex};

use api::{
    collector::{Collector, TagInfo},
    dom::{ArcDomNode, ConvertDom, DomNode},
    logging::{DURATION, EVENT, EVENT_PAGE_LOADED, STATUS_CODE},
    sender::NetworkMessage,
};
use html_parser::parser::streaming::HtmlStreamParser;
use network::web::client::WebClient;
use tokio::sync::{mpsc, oneshot};
use tracing::{Span, error, info};

use crate::network::{client::setup_new_client, thread::spawn_network_thread};

/// A collector that gathers metadata from HTML tags in a browser tab.
///
/// # Fields
/// * `in_head` - A boolean indicating whether the collector is currently processing the `<head>` section of the HTML document.
/// * `url` - The URL of the page being collected.
/// * `title` - The title of the page, if available.
/// * `external_resources` - A vector of tuples containing DOM nodes and their associated resource URLs (e.g., scripts, stylesheets).
#[derive(Default)]
pub struct TabCollector {
    in_head: bool,
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
#[derive(Clone)]
pub struct BrowserTab {
    pub id: usize,
    pub web_client: Arc<Mutex<WebClient>>,
    pub network_sender: mpsc::UnboundedSender<NetworkMessage>,
    pub temp_url: String,
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

        let client = Arc::new(Mutex::new(client.unwrap()));

        let span = tracing::info_span!("BrowserTab", id = id);

        let network_sender = {
            let _enter = span.enter();
            spawn_network_thread(client.clone(), span.clone())
        };

        let url_clone = url.clone();

        BrowserTab {
            id,
            web_client: client,
            network_sender,
            url,
            temp_url: url_clone,
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
        self.temp_url = url.clone();
        self.url = url.clone();

        tokio::spawn(async move {
            let _enter = span_clone.enter();

            match response_rx.await {
                Ok(network_response) => match network_response {
                    Ok(resp) => {
                        let content = resp.text().await;
                        if let Err(err) = content {
                            error!("Failed to read response body: {}", err);
                            return;
                        }

                        let html = content.unwrap();

                        let parser = HtmlStreamParser::new(html.as_bytes(), None);

                        let parsed = parser.parse(Some(TabCollector {
                            in_head: false,
                            url: url_clone,
                            title: Some("Blank".to_string()),
                            favicons: Vec::new(),
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
