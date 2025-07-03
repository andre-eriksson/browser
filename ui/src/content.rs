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
            let renderer = renderer.clone();

            // Get the HTML content first to avoid borrowing conflicts
            let html_content = if let Ok(html) = tab.html_content.lock() {
                html.clone()
            } else {
                return; // Exit early if we can't get the HTML content
            };

            // Scrollable area for HTML content
            ScrollArea::vertical()
                .auto_shrink(false)
                .drag_to_scroll(false)
                .show(ui, |ui| match &html_content {
                    ConcurrentDomNode::Document(children) => {
                        display_child_elements(metadata_clone, ui, children, renderer, tab);
                    }
                    _ => {
                        ui.label("No HTML content loaded.");
                    }
                });
        });
}

fn display_child_elements(
    metadata_clone: Arc<Mutex<TabCollector>>,
    ui: &mut egui::Ui,
    children: &Vec<Arc<Mutex<ConcurrentDomNode>>>,
    renderer: Arc<Mutex<HtmlRenderer>>,
    tab: &mut BrowserTab,
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
                        tab,
                    );
                    break;
                }

                if element.tag_name.eq_ignore_ascii_case("body") {
                    let mut renderer = renderer.lock().unwrap();
                    renderer.display(ui, &metadata_clone, &element, tab);
                }
            }
            _ => {}
        }
    }
}
