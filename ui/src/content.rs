use api::dom::DomNode;
use egui::{CentralPanel, Color32, Margin, ScrollArea, Stroke};

use crate::{
    html::ui::{display_body, find_and_display_body},
    topbar::BrowserTab,
};

/// Renders the main content area of the browser UI, displaying HTML content.
pub fn render_content(ctx: &egui::Context, tab: &mut BrowserTab) {
    CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(Color32::from_rgb(255, 255, 255))
                .stroke(Stroke::new(1.0, Color32::from_rgb(200, 200, 200)))
                .inner_margin(Margin::same(10)),
        )
        .show(ctx, |ui| {
            let metadata_clone = tab.metadata.clone();

            if let Ok(html) = tab.html_content.lock() {
                // Scrollable area for HTML content
                ScrollArea::vertical()
                    .auto_shrink(false)
                    .drag_to_scroll(false)
                    .show(ui, |ui| match &*html {
                        DomNode::Document(children) => {
                            // Look for the body element specifically
                            let mut found_body = false;
                            for child in children {
                                match child.lock().unwrap().clone() {
                                    DomNode::Element(element) => {
                                        if element.tag_name.as_str() == "html" {
                                            find_and_display_body(ui, &metadata_clone, &element);
                                            found_body = true;
                                        } else if element.tag_name.as_str() == "body" {
                                            display_body(ui, &metadata_clone, &element, 0);
                                            found_body = true;
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            if !found_body {
                                ui.label("No body element found in HTML document.");
                            }
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
