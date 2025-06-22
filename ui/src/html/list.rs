use api::dom::{AtomicDomNode, AtomicElement};

use crate::html::ui::{collect_text_content, MAX_DEPTH};

/// Displays a list (ul or ol) with proper formatting
pub fn display_list(ui: &mut egui::Ui, element: &AtomicElement, current_depth: usize) {
    if current_depth > MAX_DEPTH {
        ui.label(format!("{}... (depth limit reached)", element.tag_name));
        return;
    }

    let mut item_count = 0;
    for child in &element.children {
        match child {
            AtomicDomNode::Element(child_element) => {
                if child_element.tag_name.as_str() == "li" {
                    item_count += 1;
                    let mut text_content = String::new();
                    collect_text_content(&mut text_content, child_element);
                    let trimmed = text_content.trim();
                    if !trimmed.is_empty() {
                        let prefix = match element.tag_name.as_str() {
                            "ol" => format!("{}. ", item_count),
                            _ => "• ".to_string(),
                        };
                        ui.label(format!("{}{}", prefix, trimmed));
                    }
                }
            }
            _ => {}
        }
    }
}
