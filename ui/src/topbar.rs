use api::{collector::DefaultCollector, dom::AtomicDomNode, sender::NetworkMessage};
use egui::{Color32, Margin, TopBottomPanel};
use html_parser::parser::streaming::HtmlStreamParser;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use tracing::error;

/// Renders the top bar of the browser UI, including a URL input field and a button to load the page.
pub fn render_top_bar(
    url: &mut String,
    status_code: &mut Arc<Mutex<String>>,
    html_content: &Arc<Mutex<AtomicDomNode>>,
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
                ui.label("Enter URL:");
            });

            // URL input field
            ui.horizontal(|ui| {
                ui.add_sized(
                    [ui.available_width() - 50.0, 20.0],
                    egui::TextEdit::singleline(url),
                );
                let button = ui.add(egui::Button::new("Load"));
                if button.clicked() {
                    let (response_tx, response_rx) = oneshot::channel();

                    network_sender
                        .send(NetworkMessage::InitializePage {
                            full_url: url.clone(),
                            response: response_tx,
                        })
                        .expect("Failed to send InitializePage message");

                    // TODO: Improve performance here.
                    let html_content_clone = html_content.clone();
                    let status_code_clone = status_code.clone();

                    tokio::spawn(async move {
                        match response_rx.await {
                            Ok(network_response) => match network_response {
                                Ok(html) => {
                                    let parser = HtmlStreamParser::builder(html.as_bytes())
                                        .collector(DefaultCollector::default())
                                        .build();

                                    let parsed = parser.parse();

                                    match parsed {
                                        Ok(result) => {
                                            let dom_tree = result.dom_tree;
                                            let mut html_content =
                                                html_content_clone.lock().unwrap();
                                            *html_content = AtomicDomNode::from(&dom_tree);
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

            let status = status_code.lock().unwrap();
            if *status != "200 OK" && !status.is_empty() {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!("Status Code: {}", status));
                });
            }
        });
}
