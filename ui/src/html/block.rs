use api::dom::AtomicElement;
use egui::RichText;

use crate::html::{
    inline::{collect_inline_segments, display_inline_segments, has_mixed_inline_content},
    ui::{MAX_DEPTH, collect_text_content},
};

/// Displays an element as a block, breaking to a new line
pub fn display_element_block(ui: &mut egui::Ui, element: &AtomicElement, current_depth: usize) {
    if current_depth > MAX_DEPTH {
        ui.label(format!("{}... (depth limit reached)", element.tag_name));
        return;
    } // Check if this element has mixed inline content (text + links/formatting)
    if has_mixed_inline_content(element) {
        // For headings with mixed content, we need to apply styling to the entire content
        let heading_size = match element.tag_name.as_str() {
            "h1" => Some(24.0),
            "h2" => Some(20.0),
            "h3" => Some(18.0),
            "h4" => Some(16.0),
            "h5" => Some(14.0),
            "h6" => Some(12.0),
            _ => None,
        };

        // Use the segment-based approach for all mixed inline content
        let segments = collect_inline_segments(element);
        display_inline_segments(ui, &segments, heading_size);
        return;
    }

    // Fallback to old behavior for simple text content
    let mut text_content = String::new();
    collect_text_content(&mut text_content, element);

    let trimmed = text_content.trim();
    if !trimmed.is_empty() {
        // Apply different styling based on element type
        let text = match element.tag_name.as_str() {
            "h1" => RichText::new(trimmed).size(24.0).strong(),
            "h2" => RichText::new(trimmed).size(20.0).strong(),
            "h3" => RichText::new(trimmed).size(18.0).strong(),
            "h4" => RichText::new(trimmed).size(16.0).strong(),
            "h5" => RichText::new(trimmed).size(14.0).strong(),
            "h6" => RichText::new(trimmed).size(12.0).strong(),
            _ => RichText::new(trimmed),
        };
        ui.label(text);
    }
}
