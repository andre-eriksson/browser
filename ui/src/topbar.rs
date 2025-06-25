use api::{
    collector::{Collector, TagInfo},
    dom::{DomNode, SharedDomNode},
    sender::NetworkMessage,
};
use egui::{Color32, Margin, TopBottomPanel};
use html_parser::parser::streaming::HtmlStreamParser;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use tracing::error;

pub type TabMetadata = Arc<Mutex<(Option<String>, Option<Vec<(SharedDomNode, Arc<str>)>>)>>;

/// A collector that gathers metadata from HTML tags in a browser tab.
///
/// # Fields
/// * `url` - The URL of the page being collected.
/// * `title` - The title of the page, if available.
/// * `external_resources` - A vector of tuples containing DOM nodes and their associated resource URLs (e.g., scripts, stylesheets).
#[derive(Default)]
pub struct TabCollector {
    url: String,
    pub title: Option<String>,
    pub external_resources: Option<Vec<(SharedDomNode, Arc<str>)>>,
}

impl Collector for TabCollector {
    type Output = (Option<String>, Option<Vec<(SharedDomNode, Arc<str>)>>);

    fn collect(&mut self, tag: &TagInfo) {
        if let Some(external_resources) = &mut self.external_resources {
            if let Some(href) = tag.attributes.get("href") {
                if tag.tag_name == "a" {
                    return; // Skip anchor tags for href collection
                }

                external_resources.push((tag.dom_node.clone(), Arc::from(href.as_str())));
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

                external_resources.push((tag.dom_node.clone(), Arc::from(resolved_src)));
            }
        }

        if tag.tag_name == "title" {
            if let DomNode::Text(ref text) = *tag.dom_node.lock().unwrap() {
                self.title = Some(text.clone());
            }
        }
    }

    fn into_result(self) -> Self::Output {
        (self.title, self.external_resources)
    }
}

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
    pub html_content: SharedDomNode,
    pub metadata: TabMetadata,
}

/// Renders the top bar of the browser UI, including a URL input field and a button to load the page.
pub fn render_top_bar(
    ctx: &egui::Context,
    tab: &mut BrowserTab,
    network_sender: &mpsc::UnboundedSender<NetworkMessage>,
) {
    TopBottomPanel::top("browser_top_panel")
        .frame(
            egui::Frame::new()
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(200, 200, 200)))
                .shadow(egui::epaint::Shadow {
                    spread: 0,
                    offset: [0, 2],
                    blur: 5,
                    color: Color32::from_black_alpha(50),
                })
                .fill(Color32::from_rgb(240, 240, 240))
                .inner_margin(Margin::same(10)),
        )
        .show(ctx, |ui| {
            // Tabs
            ui.horizontal(|ui| {
                // TODO: Render tabs, and <head> content like title, meta tags, etc.
                let _ = ui.button(
                    tab.metadata
                        .lock()
                        .unwrap()
                        .0
                        .clone()
                        .unwrap_or_else(|| "Blank".to_string()),
                );
                ui.separator();
                let _ = ui.button("+");
            });

            // URL input field
            ui.horizontal(|ui| {
                ui.add_sized(
                    [ui.available_width() - 50.0, 20.0],
                    egui::TextEdit::singleline(&mut tab.url),
                );
                let button = ui.add(egui::Button::new("Load"));
                if button.clicked() {
                    let (response_tx, response_rx) = oneshot::channel();

                    network_sender
                        .send(NetworkMessage::InitializePage {
                            full_url: tab.url.clone(),
                            response: response_tx,
                        })
                        .expect("Failed to send InitializePage message");

                    // TODO: Improve performance here, viewport scrolling, etc.
                    let url_clone = tab.url.clone();
                    let html_content_clone = tab.html_content.clone();
                    let status_code_clone = tab.status_code.clone();
                    let metadata_clone = tab.metadata.clone();

                    tokio::spawn(async move {
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

                                            let mut html_content =
                                                html_content_clone.lock().unwrap();
                                            *html_content = dom_tree;
                                        }
                                        Err(err) => {
                                            eprintln!("Parsing error: {}", err);
                                        }
                                    }

                                    *status_code_clone.lock().unwrap() = "200 OK".to_string();
                                }
                                Err(err) => {
                                    *status_code_clone.lock().unwrap() = err.to_string();
                                }
                            },

                            Err(_) => {
                                error!("Failed to receive response from network thread.");
                            }
                        }
                    });
                }
            });

            let status = tab.status_code.lock().unwrap();
            if *status != "200 OK" && !status.is_empty() {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!("Status Code: {}", status));
                });
            }
        });
}
