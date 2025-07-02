use std::sync::{Arc, Mutex};

use api::logging::{EVENT, EVENT_NEW_TAB, EVENT_TAB_CLOSED};
use egui::{Color32, Margin, TopBottomPanel};
use tracing::info;

use crate::{api::tabs::BrowserTab, network::client::setup_new_client};

/// Renders the top bar of the browser UI, including a URL input field and a button to load the page.
pub fn render_top_bar(
    ctx: &egui::Context,
    tabs: Arc<Mutex<Vec<BrowserTab>>>,
    current_tab: Arc<Mutex<usize>>,
    next_tab_id: Arc<Mutex<usize>>,
    close_tab_fn: &mut dyn FnMut(usize),
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
            let mut tab_to_close = None;
            // Tabs
            ui.horizontal(|ui| {
                let mut tabs_guard = tabs.lock().unwrap();
                let mut current_tab_guard = current_tab.lock().unwrap();

                // TODO: Render tabs, and <head> content like title, meta tags, etc.
                for (i, tab) in tabs_guard.iter_mut().enumerate() {
                    let tab_label = if let Some(title) = &tab.metadata.lock().unwrap().title {
                        title.clone()
                    } else {
                        "Untitled".to_string()
                    };

                    let color = if *current_tab_guard == i {
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
                        *current_tab_guard = i; // Update the current tab index
                    }

                    if tab_button.secondary_clicked() {
                        tab_to_close = Some(tab.id);
                    }
                }

                ui.separator();
                let new_tab = ui.button("+");
                if new_tab.clicked() {
                    let client = setup_new_client();

                    if let Err(e) = client {
                        eprintln!("Failed to create new client: {}", e);
                        return;
                    }

                    // Get the next unique tab ID
                    let tab_id = {
                        let mut next_id = next_tab_id.lock().unwrap();
                        let id = *next_id;
                        *next_id += 1;
                        id
                    };

                    let new_browser_tab =
                        BrowserTab::new(tab_id, "http://localhost:8000/image.html".to_string());

                    tabs_guard.push(new_browser_tab);
                    if tabs_guard.len() > 0 {
                        *current_tab_guard = tabs_guard.len() - 1; // Switch to the new tab
                    }
                    info!({ EVENT } = EVENT_NEW_TAB, tab_id = tab_id);
                }
            });

            if let Some(tab_id) = tab_to_close {
                info!({ EVENT } = EVENT_TAB_CLOSED, tab_id = tab_id);
                close_tab_fn(tab_id);
            }

            ui.separator();

            let mut tabs_guard = tabs.lock().unwrap();
            let current_tab_guard = current_tab.lock().unwrap();
            let tab = &mut tabs_guard[*current_tab_guard];
            // URL input field
            ui.horizontal(|ui| {
                ui.add_sized(
                    [ui.available_width() - 50.0, 20.0],
                    egui::TextEdit::singleline(&mut tab.url),
                );
                let button = ui.add(egui::Button::new("Load"));
                if button.clicked() {
                    tab.navigate_to(tab.url.clone());
                }
            });
        });
}
