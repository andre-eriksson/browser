use api::dom::{DomNode, Element};
use egui::{Color32, RichText};

use crate::html::ui::collect_text_content;

/// Collects inline content with formatting information
///
/// # Fields
/// * `text`: The text content of the inline element.
///  * `is_link`: Indicates if the text is part of a link.
///  * `is_bold`: Indicates if the text is bold.
///  * `is_italic`: Indicates if the text is italic.
///  * `is_superscript`: Indicates if the text is superscript.
#[derive(Debug, Clone)]
pub struct InlineSegment {
    text: String,
    is_link: bool,
    is_bold: bool,
    is_italic: bool,
    is_superscript: bool,
}

/// Displays inline elements on the same line using egui's horizontal layout
pub fn display_inline_elements(ui: &mut egui::Ui, elements: &[&Element]) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0; // Remove spacing between elements
        for element in elements {
            match element.tag_name.as_str() {
                "a" => {
                    let mut text_content = String::new();
                    collect_text_content(&mut text_content, element);
                    let trimmed = text_content.trim();
                    if !trimmed.is_empty() {
                        ui.label(RichText::new(trimmed).color(Color32::BLUE).underline());
                    }
                }
                "span" | "strong" | "em" | "b" | "i" | "sup" => {
                    let mut text_content = String::new();
                    collect_text_content(&mut text_content, element);
                    let trimmed = text_content.trim();
                    if !trimmed.is_empty() {
                        let text = match element.tag_name.as_str() {
                            "strong" | "b" => RichText::new(trimmed).strong(),
                            "em" | "i" => RichText::new(trimmed).italics(),
                            "sup" => RichText::new(trimmed).size(10.0),
                            _ => RichText::new(trimmed),
                        };
                        ui.label(text);
                    }
                }
                _ => {
                    let mut text_content = String::new();
                    collect_text_content(&mut text_content, element);
                    let trimmed = text_content.trim();
                    if !trimmed.is_empty() {
                        ui.label(trimmed);
                    }
                }
            }
        }
    });
}

/// Checks if an element contains only inline content
pub fn has_only_inline_children(element: &Element) -> bool {
    for child in &element.children {
        match child.lock().unwrap().clone() {
            DomNode::Element(child_element) => {
                match child_element.tag_name.as_str() {
                    "span" | "a" | "strong" | "em" | "b" | "i" | "sup" => continue,
                    "style" | "script" | "link" | "meta" | "noscript" => continue,
                    _ => return false, // Found a non-inline element
                }
            }
            DomNode::Text(_) => continue,
            _ => {}
        }
    }
    true
}

/// Collects all inline segments from an element, preserving formatting
pub fn collect_inline_segments(element: &Element) -> Vec<InlineSegment> {
    let mut segments = Vec::new();
    collect_inline_segments_recursive(element, &mut segments, false, false, false, false);
    segments
}

fn collect_inline_segments_recursive(
    element: &Element,
    segments: &mut Vec<InlineSegment>,
    is_link: bool,
    is_bold: bool,
    is_italic: bool,
    is_superscript: bool,
) {
    for child in &element.children {
        match child.lock().unwrap().clone() {
            DomNode::Element(child_element) => {
                match child_element.tag_name.as_str() {
                    "a" => {
                        collect_inline_segments_recursive(
                            &child_element,
                            segments,
                            true,
                            is_bold,
                            is_italic,
                            is_superscript,
                        );
                    }
                    "strong" | "b" => {
                        collect_inline_segments_recursive(
                            &child_element,
                            segments,
                            is_link,
                            true,
                            is_italic,
                            is_superscript,
                        );
                    }
                    "em" | "i" => {
                        collect_inline_segments_recursive(
                            &child_element,
                            segments,
                            is_link,
                            is_bold,
                            true,
                            is_superscript,
                        );
                    }
                    "sup" => {
                        collect_inline_segments_recursive(
                            &child_element,
                            segments,
                            is_link,
                            is_bold,
                            is_italic,
                            true,
                        );
                    }
                    "style" | "script" | "link" | "meta" | "noscript" => {
                        // Skip non-content elements
                        continue;
                    }
                    _ => {
                        // For other elements like span, continue with current formatting
                        collect_inline_segments_recursive(
                            &child_element,
                            segments,
                            is_link,
                            is_bold,
                            is_italic,
                            is_superscript,
                        );
                    }
                }
            }
            DomNode::Text(text) => {
                if !text.is_empty() {
                    segments.push(InlineSegment {
                        text: text.clone(),
                        is_link,
                        is_bold,
                        is_italic,
                        is_superscript,
                    });
                }
            }
            _ => {}
        }
    }
}

/// Displays inline segments with proper spacing
pub fn display_inline_segments(
    ui: &mut egui::Ui,
    segments: &[InlineSegment],
    base_size: Option<f32>,
) {
    if segments.is_empty() {
        return;
    }

    // First, normalize whitespace and group segments with same formatting
    let mut normalized_segments: Vec<InlineSegment> = Vec::new();

    for segment in segments {
        // Normalize whitespace but preserve it
        let normalized_text = segment.text.replace('\n', " ").replace('\t', " ");

        // Skip completely empty segments
        if normalized_text.trim().is_empty() && normalized_text.is_empty() {
            continue;
        }

        // Try to merge with previous segment if same formatting
        if let Some(last) = normalized_segments.last_mut() {
            if last.is_link == segment.is_link
                && last.is_bold == segment.is_bold
                && last.is_italic == segment.is_italic
                && last.is_superscript == segment.is_superscript
            {
                last.text.push_str(&normalized_text);
                continue;
            }
        }

        normalized_segments.push(InlineSegment {
            text: normalized_text,
            is_link: segment.is_link,
            is_bold: segment.is_bold,
            is_italic: segment.is_italic,
            is_superscript: segment.is_superscript,
        });
    }

    // Now render the segments
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0; // We'll handle spacing manually

        for (i, segment) in normalized_segments.iter().enumerate() {
            let text = &segment.text;

            // Skip segments that are only whitespace, unless they're between two non-whitespace segments
            if text.trim().is_empty() {
                if i > 0 && i < normalized_segments.len() - 1 {
                    // This is whitespace between segments, render a space
                    ui.label(" ");
                }
                continue;
            }

            let size = if segment.is_superscript {
                base_size.map(|s| s * 0.7).unwrap_or(10.0)
            } else {
                base_size.unwrap_or(12.0)
            };

            let mut rich_text = RichText::new(text).size(size);

            if segment.is_bold || base_size.is_some() {
                rich_text = rich_text.strong();
            }
            if segment.is_italic {
                rich_text = rich_text.italics();
            }
            if segment.is_link {
                rich_text = rich_text.color(Color32::BLUE).underline();
            }

            ui.label(rich_text);
        }
    });
}

/// Checks if an element has mixed inline content (text + inline elements)
pub fn has_mixed_inline_content(element: &Element) -> bool {
    let mut has_text = false;
    let mut has_inline_elements = false;

    for child in &element.children {
        match child.lock().unwrap().clone() {
            DomNode::Element(child_element) => {
                match child_element.tag_name.as_str() {
                    "span" | "a" | "strong" | "em" | "b" | "i" | "sup" => {
                        has_inline_elements = true;
                    }
                    "style" | "script" | "link" | "meta" | "noscript" => {
                        continue; // Skip non-content elements
                    }
                    _ => return false, // Found a block element
                }
            }
            DomNode::Text(text) => {
                if !text.trim().is_empty() {
                    has_text = true;
                }
            }
            _ => {}
        }
    }

    has_inline_elements || has_text
}
