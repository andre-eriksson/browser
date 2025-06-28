use std::sync::{Arc, Mutex};

use api::dom::DomNode;
use egui::{CentralPanel, Color32, ScrollArea};

use crate::{
    html::ui::HtmlRenderer,
    topbar::{BrowserTab, TabCollector},
};

/// Renders the main content area of the browser UI, displaying HTML content.
pub fn render_content(ctx: &egui::Context, tab: &mut BrowserTab) {
    CentralPanel::default()
        .frame(egui::Frame::new().fill(Color32::from_rgb(255, 255, 255)))
        .show(ctx, |ui| {
            let metadata_clone = tab.metadata.clone();
            let renderer_clone = tab.renderer.clone();

            if let Ok(html) = tab.html_content.lock() {
                // Scrollable area for HTML content
                ScrollArea::vertical()
                    .auto_shrink(false)
                    .drag_to_scroll(false)
                    .show(ui, |ui| match &*html {
                        DomNode::Document(children) => {
                            display_child_elements(metadata_clone, ui, children, renderer_clone);
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
    children: &Vec<Arc<Mutex<DomNode>>>,
    renderer: Arc<Mutex<HtmlRenderer>>,
) {
    for child in children {
        match child.lock().unwrap().clone() {
            DomNode::Element(element) => {
                if element.tag_name.eq_ignore_ascii_case("html") {
                    display_child_elements(metadata_clone, ui, &element.children, renderer.clone());
                    break;
                }

                if element.tag_name.eq_ignore_ascii_case("body") {
                    let mut renderer = renderer.lock().unwrap();
                    renderer.display(ui, &metadata_clone, &element);
                }
            }
            _ => {}
        }
    }
}
