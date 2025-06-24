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

#[derive(Default)]
pub struct TabCollector {
    pub title: Option<String>,
}

impl Collector for TabCollector {
    type Output = Option<String>;

    fn collect(&mut self, tag: &TagInfo) {
        if tag.tag_name != "title" {
            return;
        }

        if let DomNode::Text(ref text) = *tag.dom_node.lock().unwrap() {
            self.title = Some(text.clone());
        }
    }

    fn into_result(self) -> Self::Output {
        self.title
    }
}

pub struct BrowserTab {
    pub url: String,
    pub status_code: Arc<Mutex<String>>,
    pub title: Arc<Mutex<String>>,
    pub html_content: SharedDomNode,
}

/// Renders the top bar of the browser UI, including a URL input field and a button to load the page.
pub fn render_top_bar(
    tab: &mut BrowserTab,
    network_sender: &mpsc::UnboundedSender<NetworkMessage>,
    ctx: &egui::Context,
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
                let _ = ui.button(tab.title.lock().unwrap().clone());
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
                    let html_content_clone = tab.html_content.clone();
                    let title_clone = tab.title.clone();
                    let status_code_clone = tab.status_code.clone();

                    tokio::spawn(async move {
                        match response_rx.await {
                            Ok(network_response) => match network_response {
                                Ok(html) => {
                                    let parser = HtmlStreamParser::new(html.as_bytes(), None);

                                    let parsed = parser.parse(Some(TabCollector {
                                        title: Some("Blank".to_string()),
                                        ..Default::default()
                                    }));

                                    match parsed {
                                        Ok(result) => {
                                            let dom_tree = result.dom_tree;

                                            //println!("Parsed DOM Tree: {:?}", dom_tree);

                                            // TODO: Process the metadata, e.g. event queue, etc.

                                            let title_text = result
                                                .metadata
                                                .unwrap_or_else(|| "Untitled".to_string());
                                            *title_clone.lock().unwrap() = title_text;

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
