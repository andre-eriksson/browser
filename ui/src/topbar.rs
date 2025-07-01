use api::{
    dom::{ArcDomNode, ConvertDom},
    sender::NetworkMessage,
};
use egui::{Color32, Margin, TopBottomPanel};
use html_parser::parser::streaming::HtmlStreamParser;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use tracing::error;

use crate::api::tabs::{BrowserTab, TabCollector};

/// Renders the top bar of the browser UI, including a URL input field and a button to load the page.
pub fn render_top_bar(
    ctx: &egui::Context,
    network_sender: &mpsc::UnboundedSender<NetworkMessage>,
    tabs: &mut Vec<BrowserTab>,
    current_tab: &mut usize,
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
                for (i, tab) in tabs.iter_mut().enumerate() {
                    let tab_label = if let Some(title) = &tab.metadata.lock().unwrap().title {
                        title.clone()
                    } else {
                        "Untitled".to_string()
                    };

                    let color = if *current_tab == i {
                        Color32::from_rgb(200, 200, 255) // Highlight the current tab
                    } else {
                        Color32::from_rgb(220, 220, 220) // Default color for other tabs
                    };

                    let tab_button = ui.add(
                        egui::Button::new(tab_label)
                            .fill(color)
                            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(180, 180, 180))),
                    );

                    if tab_button.clicked() {
                        *current_tab = i; // Update the current tab index
                    }
                }

                ui.separator();
                let new_tab = ui.button("+");
                if new_tab.clicked() {
                    // Create a new tab with a default URL
                    let new_browser_tab = BrowserTab {
                        url: "http://localhost:8000/test.html".to_string(),
                        status_code: Arc::new(Mutex::new("200 OK".to_string())),
                        html_content: Default::default(),
                        metadata: Default::default(),
                    };
                    tabs.push(new_browser_tab);
                }
            });

            ui.separator();

            let tab = &mut tabs[*current_tab];
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
                    let html_content_clone: ArcDomNode = tab.html_content.clone();
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

                                            let mut html_content = html_content_clone.lock().unwrap();
                                            *html_content = dom_tree.convert().lock().unwrap().clone();
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
        });
}
