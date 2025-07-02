use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use api::dom::ConcurrentDomNode;
use egui::{CentralPanel, Color32, ColorImage, ScrollArea};

use crate::{
    api::tabs::{BrowserTab, TabCollector},
    html::ui::HtmlRenderer,
    network::loader::NetworkLoader,
};

/// Renders the main content area of the browser UI, displaying HTML content.
pub fn render_content(
    ctx: &egui::Context,
    cache: &Arc<Mutex<HashMap<String, Arc<ColorImage>>>>,
    renderer: &Arc<Mutex<HtmlRenderer>>,
    tab: &mut BrowserTab,
) {
    ctx.add_image_loader(Arc::new(NetworkLoader {
        network_sender: tab.network_sender.clone(),
        cache: cache.clone(),
    }));
    CentralPanel::default()
        .frame(egui::Frame::new().fill(Color32::from_rgb(255, 255, 255)))
        .show(ctx, |ui| {
            let metadata_clone = tab.metadata.clone();
            let url = tab.url.clone();
            let renderer = renderer.clone();

            if let Ok(html) = tab.html_content.lock() {
                // Scrollable area for HTML content
                ScrollArea::vertical()
                    .auto_shrink(false)
                    .drag_to_scroll(false)
                    .show(ui, |ui| match &*html {
                        ConcurrentDomNode::Document(children) => {
                            display_child_elements(metadata_clone, ui, children, renderer, url);
                        }
                        _ => {
                            ui.label("No HTML content loaded.");
                        }
                    });
            } else {
                ui.label("HTML content would be displayed here.");
            }
        });
}

fn display_child_elements(
    metadata_clone: Arc<Mutex<TabCollector>>,
    ui: &mut egui::Ui,
    children: &Vec<Arc<Mutex<ConcurrentDomNode>>>,
    renderer: Arc<Mutex<HtmlRenderer>>,
    url: String,
) {
    for child in children {
        match child.lock().unwrap().clone() {
            ConcurrentDomNode::Element(element) => {
                if element.tag_name.eq_ignore_ascii_case("html") {
                    display_child_elements(
                        metadata_clone,
                        ui,
                        &element.children,
                        renderer.clone(),
                        url.clone(),
                    );
                    break;
                }

                if element.tag_name.eq_ignore_ascii_case("body") {
                    let mut renderer = renderer.lock().unwrap();
                    renderer.display(ui, &metadata_clone, &element, url.as_str());
                }
            }
            _ => {}
        }
    }
}
