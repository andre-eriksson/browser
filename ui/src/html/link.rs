use api::dom::AtomicElement;
use egui::{Color32, RichText};

use crate::html::ui::collect_text_content;

/// Displays a link element
pub fn display_link(ui: &mut egui::Ui, element: &AtomicElement) {
    let mut text_content = String::new();
    collect_text_content(&mut text_content, element);
    let trimmed = text_content.trim();
    if !trimmed.is_empty() {
        ui.label(RichText::new(trimmed).color(Color32::BLUE).underline());
    }
}
