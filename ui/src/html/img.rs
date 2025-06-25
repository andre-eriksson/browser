use std::borrow::Cow;

use api::dom::{DomNode, Element};
use egui::{ImageSource, RichText};

use crate::{html::inline::display_inline_elements, topbar::TabMetadata};

/// Displays an image element, fetching the full URL from metadata
pub fn display_image(ui: &mut egui::Ui, metadata: &TabMetadata, element: &Element) {
    // Get the full url from the metadata 1 hashmap element
    let metadata = metadata.lock().unwrap();
    let external_resources = &metadata.1;
    let mut full_src = "";

    let width = element
        .attributes
        .get("width")
        .and_then(|w| w.parse::<f32>().ok());
    let height = element
        .attributes
        .get("height")
        .and_then(|h| h.parse::<f32>().ok());
    let alt = element.attributes.get("alt").map(|s| s.as_str());

    if let Some(external_resource) = external_resources {
        for (key, value) in external_resource.iter() {
            match key.lock().unwrap().clone() {
                DomNode::Element(ref ele) => {
                    if ele.id == element.id {
                        full_src = value;
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    let image = egui::Image::new(ImageSource::Uri(Cow::Borrowed(full_src)))
        .max_size(egui::vec2(width.unwrap_or(100.0), height.unwrap_or(100.0)));

    if let Some(alt_text) = alt {
        ui.label(egui::RichText::new(alt_text).color(egui::Color32::GRAY));
    }

    ui.add(image);
}

/// Displays inline segments with images, preserving formatting and layout
pub fn display_inline_segments_with_images(
    ui: &mut egui::Ui,
    metadata: &TabMetadata,
    element: &Element,
    base_size: Option<f32>,
) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0; // Small spacing between elements

        for child in &element.children {
            match child.lock().unwrap().clone() {
                DomNode::Element(child_element) => {
                    match child_element.tag_name.as_str() {
                        "img" => {
                            display_image(ui, metadata, &child_element);
                        }
                        "a" => {
                            // Handle links that might contain images or text
                            display_inline_elements(ui, metadata, &[&child_element]);
                        }
                        "strong" | "b" | "em" | "i" | "sup" | "span" => {
                            // Handle formatted text elements
                            display_inline_elements(ui, metadata, &[&child_element]);
                        }
                        "style" | "script" | "link" | "meta" | "noscript" => {
                            // Skip non-content elements
                            continue;
                        }
                        _ => {
                            // Handle other inline elements
                            display_inline_elements(ui, metadata, &[&child_element]);
                        }
                    }
                }
                DomNode::Text(text) => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        let size = base_size.unwrap_or(12.0);
                        let rich_text = RichText::new(trimmed).size(size);
                        ui.label(rich_text);
                    }
                }
                _ => {}
            }
        }
    });
}
