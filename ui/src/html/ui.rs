use api::dom::{AtomicDomNode, AtomicElement};

use crate::html::{
    block::display_element_block,
    div::display_div,
    inline::{display_inline_elements, has_only_inline_children},
    link::display_link,
    list::display_list,
};

pub const MAX_DEPTH: usize = 25;

/// Finds the body element within the HTML document and displays it.
pub fn find_and_display_body(ui: &mut egui::Ui, element: &AtomicElement) {
    // Look for body element within html element
    for child in &element.children {
        match child {
            AtomicDomNode::Element(child_element) => {
                if child_element.tag_name.as_str() == "body" {
                    display_body(ui, child_element, 0);
                    return;
                }
            }
            _ => {}
        }
    }
}

/// Displays the body element and its children, handling depth limits
pub fn display_body(ui: &mut egui::Ui, element: &AtomicElement, current_depth: usize) {
    if current_depth > MAX_DEPTH {
        ui.label(format!("{}... (depth limit reached)", element.tag_name));
        return;
    }

    // Only render if this is a body element
    if element.tag_name.as_str() != "body" {
        return;
    }
    for child in &element.children {
        match child {
            AtomicDomNode::Element(child_element) => match child_element.tag_name.as_str() {
                "div" => display_div(ui, child_element, current_depth + 1),
                "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    display_element_block(ui, child_element, current_depth + 1);
                }
                "header" | "main" | "nav" | "section" | "article" | "aside" | "footer" => {
                    // Handle semantic HTML elements
                    display_element(ui, child_element, current_depth + 1);
                }
                "ul" | "ol" => {
                    display_list(ui, child_element, current_depth + 1);
                }
                "a" => {
                    display_link(ui, child_element);
                }
                "style" | "script" | "link" | "meta" | "noscript" => {
                    // Skip non-content elements
                    continue;
                }
                _ => display_element(ui, child_element, current_depth + 1),
            },
            AtomicDomNode::Text(text) => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    ui.label(trimmed);
                }
            }
            _ => {}
        }
    }
}

/// Recursively collects all text content from an element and its inline children
pub fn collect_text_content(text_content: &mut String, element: &AtomicElement) {
    for child in &element.children {
        match child {
            AtomicDomNode::Element(child_element) => {
                match child_element.tag_name.as_str() {
                    // For inline elements, collect their text content
                    "span" | "a" | "strong" | "em" | "b" | "i" | "sup" => {
                        collect_text_content(text_content, child_element);
                    }
                    "style" | "script" | "link" | "meta" | "noscript" => {
                        // Skip non-content elements
                        continue;
                    }
                    // For block elements, they should be handled separately
                    "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {}
                    _ => {
                        // For other elements, collect their text content
                        collect_text_content(text_content, child_element);
                    }
                }
            }
            AtomicDomNode::Text(text) => {
                text_content.push_str(text);
            }
            _ => {}
        }
    }
}

/// Displays an HTML element, handling different types of elements and depth limits
pub fn display_element(ui: &mut egui::Ui, element: &AtomicElement, current_depth: usize) {
    if current_depth > MAX_DEPTH {
        ui.label(format!("{}... (depth limit reached)", element.tag_name));
        return;
    }

    // Handle body elements specially
    if element.tag_name.as_str() == "body" {
        display_body(ui, element, current_depth);
        return;
    }

    // Check if this element contains only inline children and should be rendered horizontally
    if has_only_inline_children(element) {
        let inline_elements: Vec<&AtomicElement> = element
            .children
            .iter()
            .filter_map(|child| match child {
                AtomicDomNode::Element(child_element) => match child_element.tag_name.as_str() {
                    "style" | "script" | "link" | "meta" | "noscript" => None,
                    _ => Some(child_element),
                },
                _ => None,
            })
            .collect();

        if !inline_elements.is_empty() {
            display_inline_elements(ui, &inline_elements);
        }

        // Also handle any text nodes
        for child in &element.children {
            if let AtomicDomNode::Text(text) = child {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    ui.label(trimmed);
                }
            }
        }
        return;
    }

    if !element.children.is_empty() {
        for child in &element.children {
            match child {
                AtomicDomNode::Element(child_element) => {
                    match child_element.tag_name.as_str() {
                        "body" => display_body(ui, child_element, current_depth + 1),
                        "div" => display_div(ui, child_element, current_depth + 1),
                        "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                            display_element_block(ui, child_element, current_depth + 1);
                        }
                        "header" | "main" | "nav" | "section" | "article" | "aside" | "footer" => {
                            // Handle semantic HTML elements recursively
                            display_element(ui, child_element, current_depth + 1);
                        }
                        "ul" | "ol" => {
                            display_list(ui, child_element, current_depth + 1);
                        }
                        "a" => {
                            display_link(ui, child_element);
                        }
                        "style" | "script" | "link" | "meta" | "noscript" => {
                            // Skip non-content elements
                            continue;
                        }
                        "span" | "strong" | "em" | "b" | "i" | "sup" => {
                            // Inline elements should only be handled within their parent context
                            // If we reach here, treat them as having their text content
                            let mut text_content = String::new();
                            collect_text_content(&mut text_content, child_element);
                            let trimmed = text_content.trim();
                            if !trimmed.is_empty() {
                                ui.label(trimmed);
                            }
                        }
                        _ => {
                            //ui.label(format!("<{}>", child_element.tag_name));
                            display_element(ui, child_element, current_depth + 1);
                        }
                    }
                }
                AtomicDomNode::Text(text) => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        ui.label(trimmed);
                    }
                }
                _ => {}
            }
        }
    }
}
