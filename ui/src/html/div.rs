use api::dom::{DomNode, Element};
use egui::{Color32, RichText};

use crate::html::{
    block::display_element_block,
    link::display_link,
    list::display_list,
    ui::{MAX_DEPTH, display_element},
};

/// Display a div element with its children
pub fn display_div(ui: &mut egui::Ui, element: &Element, current_depth: usize) {
    if current_depth > MAX_DEPTH {
        ui.label(format!("{}... (depth limit reached)", element.tag_name));
        return;
    }

    for child in &element.children {
        match child.lock().unwrap().clone() {
            DomNode::Element(child_element) => {
                match child_element.tag_name.as_str() {
                    "div" => {
                        display_div(ui, &child_element, current_depth + 1);
                    }
                    "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        display_element_block(ui, &child_element, current_depth + 1);
                    }
                    "header" | "main" | "nav" | "section" | "article" | "aside" | "footer" => {
                        display_element(ui, &child_element, current_depth + 1);
                    }
                    "ul" | "ol" => {
                        display_list(ui, &child_element, current_depth + 1);
                    }
                    "a" => {
                        display_link(ui, &child_element);
                    }
                    "style" | "script" | "link" | "meta" | "noscript" => {
                        // Skip non-content elements
                        continue;
                    }
                    _ => {
                        display_element(ui, &child_element, current_depth + 1);
                    }
                }
            }
            DomNode::Text(text) => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    ui.label(RichText::new(trimmed).color(Color32::BLACK));
                }
            }
            _ => {}
        }
    }
}
