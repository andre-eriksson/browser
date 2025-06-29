use api::dom::{DomNode, Element};

use crate::topbar::TabMetadata;

/// A renderer for displaying HTML elements in a structured format using egui.
///
/// # Fields
/// * `max_depth` - The maximum depth to render HTML elements.
/// * `current_depth` - The current depth of the rendering process, used to limit recursion depth.
/// * `inline_buffer` - A buffer to collect inline elements before rendering them, due to egui's limitations in rendering inline elements directly.
/// * `debug` - A flag to enable debug mode, which provides additional information about the rendering process.
pub struct HtmlRenderer {
    max_depth: usize,
    current_depth: usize,
    inline_buffer: Vec<Element>,
    debug: bool,
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        HtmlRenderer {
            max_depth: 100,
            current_depth: 0,
            inline_buffer: Vec::new(),
            debug: false,
        }
    }
}

impl HtmlRenderer {
    /// Creates a new `HtmlRenderer` with the specified maximum depth and debug mode.
    ///
    /// # Arguments
    /// * `max_depth` - The maximum depth to render HTML elements.
    /// * `debug` - If true, enables debug mode which displays additional information about the rendering process.
    pub fn new(max_depth: usize, debug: bool) -> Self {
        HtmlRenderer {
            max_depth,
            current_depth: 0,
            inline_buffer: Vec::new(),
            debug,
        }
    }

    /// Displays the HTML content of a tab
    pub fn display(&mut self, ui: &mut egui::Ui, metadata: &TabMetadata, element: &Element) {
        self.current_depth = 0; // Reset depth for each new element
        self.inline_buffer.clear(); // Clear inline buffer for each new element
        self.display_body(ui, metadata, element);
    }

    fn display_body(&mut self, ui: &mut egui::Ui, metadata: &TabMetadata, element: &Element) {
        self.initialize_block_context(ui, metadata, element);
    }

    fn display_element(&mut self, ui: &mut egui::Ui, metadata: &TabMetadata, element: &Element) {
        if self.current_depth > self.max_depth {
            ui.label(format!("{}... (depth limit reached)", element.tag_name));
            return;
        }

        self.current_depth += 1;

        match element.tag_name.as_str() {
            "div" | "header" | "footer" | "main" | "section" | "article" | "aside" | "pre"
            | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "hr" => {
                if !self.inline_buffer.is_empty() {
                    self.render_inline_elements(ui);
                }

                self.initialize_block_context(ui, metadata, element);
            }
            "span" | "a" | "strong" | "em" | "i" | "b" | "u" | "code" | "small" | "sub" | "sup" => {
                self.collect_inline_elements(ui, element);
            }
            "script" | "style" => {
                // Skip script and style tags in the rendering
                if self.debug {
                    ui.label(format!(
                        "Skipping: <{}> (depth: {})",
                        element.tag_name, self.current_depth
                    ));
                }
            }
            _ => {
                // Handle unrecognized elements
                if self.debug {
                    ui.label(format!(
                        "E: <{}> (depth: {})",
                        element.tag_name, self.current_depth
                    ));
                }

                // Process children of unrecognized elements
                for child in &element.children {
                    match child.lock().unwrap().clone() {
                        DomNode::Element(child_element) => {
                            self.display_element(ui, metadata, &child_element);
                        }
                        DomNode::Text(text) => {
                            if !self.inline_buffer.is_empty() {
                                self.render_inline_elements(ui);
                            }
                            ui.label(text);
                        }
                        _ => {}
                    }
                }
            }
        }

        self.current_depth -= 1;
    }

    fn initialize_block_context(
        &mut self,
        ui: &mut egui::Ui,
        metadata: &TabMetadata,
        element: &Element,
    ) {
        if self.current_depth > self.max_depth {
            ui.label(format!("{}... (depth limit reached)", element.tag_name));
            return;
        }

        let color = match self.debug {
            true => get_depth_color(self.current_depth), // Use a color based on the current depth for debug mode
            false => egui::Color32::from_rgb(255, 255, 255), // Default white for normal mode
        };

        // TODO: Adjust margin based on element type
        let margin = match element.tag_name.as_str() {
            "body" => egui::Margin::same(8),
            "h1" => egui::Margin::symmetric(0, 8),
            "h2" => egui::Margin::symmetric(0, 7),
            "h3" => egui::Margin::symmetric(0, 6),
            "h4" => egui::Margin::symmetric(0, 7),
            "h5" => egui::Margin::symmetric(0, 8),
            "h6" => egui::Margin::symmetric(0, 9),
            _ => {
                // Base margin for other block elements
                if self.debug {
                    // More visible margin in debug mode to highlight structure via colors
                    egui::Margin::same(8)
                } else {
                    egui::Margin::symmetric(0, 4)
                }
            }
        };

        egui::Frame::new()
            .outer_margin(margin)
            .fill(color)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.x = 0.0;

                // Check if element has any n+1 text nodes

                // Semantic elements that don't "generally" contain text nodes should be rendered in regards to their children, i.e. vertically.
                // Such as <body>, <div>, <header>, <footer>, etc.
                // TODO: Handle semantic elements with text nodes that don't utilize given text elements (e.g., <p> with text nodes)
                let has_text_nodes = element
                    .children
                    .iter()
                    .any(|child| matches!(child.lock().unwrap().clone(), DomNode::Text(_)));

                if element.tag_name == "body" || !has_text_nodes {
                    ui.vertical(|ui| {
                        self.render_block_element(ui, metadata, element);
                    });
                } else {
                    ui.horizontal(|ui| {
                        self.render_block_element(ui, metadata, element);
                    });
                }
            });
    }

    fn render_block_element(
        &mut self,
        ui: &mut egui::Ui,
        metadata: &TabMetadata,
        element: &Element,
    ) {
        if self.current_depth > self.max_depth {
            ui.label(format!("{}... (depth limit reached)", element.tag_name));
            return;
        }

        if self.debug {
            ui.label(format!(
                "B: <{}> (depth: {})",
                element.tag_name, self.current_depth
            ));
        }

        if element.tag_name == "hr" {
            // Render horizontal rule with a line
            ui.separator();
            return;
        }

        // Recursively display child elements
        for child in &element.children {
            match child.lock().unwrap().clone() {
                DomNode::Element(child_element) => {
                    if is_inline_element(&child_element.tag_name) {
                        self.collect_inline_elements(ui, &child_element);
                    } else {
                        self.display_element(ui, metadata, &child_element);
                    }
                }
                DomNode::Text(text) => {
                    // Render any inline elements collected so far before rendering text
                    // For example, <p> -> TEXT -> <span> TEXT </span> -> TEXT </p>
                    if !self.inline_buffer.is_empty() {
                        self.render_inline_elements(ui);
                    }

                    match element.tag_name.as_str() {
                        "h1" => ui.label(egui::RichText::new(text).strong().size(32.0)),
                        "h2" => ui.label(egui::RichText::new(text).strong().size(24.0)),
                        "h3" => ui.label(egui::RichText::new(text).strong().size(20.0)),
                        "h4" => ui.label(egui::RichText::new(text).strong()),
                        "h5" => ui.label(egui::RichText::new(text).strong().size(10.0)),
                        "h6" => ui.label(egui::RichText::new(text).strong().size(8.0)),
                        _ => ui.label(text),
                    };
                }
                _ => {}
            }
        }

        // Render any inline elements collected so far
        // This ensures that inline elements are rendered after certain block elements such as <pre> -> <code>
        if !self.inline_buffer.is_empty() {
            self.render_inline_elements(ui);
        }
    }

    fn collect_inline_elements(&mut self, ui: &mut egui::Ui, element: &Element) {
        if self.current_depth > self.max_depth {
            ui.label(format!("{}... (depth limit reached)", element.tag_name));
            return;
        }

        // Collect inline elements into a buffer and render them later due to egui limitations in rendering
        self.inline_buffer.push(element.clone());
    }

    fn render_inline_elements(&mut self, ui: &mut egui::Ui) {
        if self.inline_buffer.is_empty() {
            return;
        }

        let color = if self.debug {
            egui::Color32::from_rgb(240, 240, 240) // Light gray for debug mode
        } else {
            egui::Color32::from_rgb(255, 255, 255) // White for normal mode
        };

        egui::Frame::new().fill(color).show(ui, |ui| {
            ui.horizontal(|ui| {
                for inline_element in &self.inline_buffer {
                    self.render_inline_element(ui, inline_element);
                }
            });
        });

        // Clear the buffer after rendering
        self.inline_buffer.clear();
    }

    fn render_inline_element(&self, ui: &mut egui::Ui, element: &Element) {
        for child in &element.children {
            match child.lock().unwrap().clone() {
                DomNode::Element(child_element) => {
                    self.render_inline_element(ui, &child_element);
                }
                DomNode::Text(text) => match element.tag_name.as_str() {
                    "code" | "pre" => {
                        ui.label(egui::RichText::new(text).monospace());
                    }
                    _ => {
                        ui.label(text);
                    }
                },
                _ => {}
            }
        }
    }
}

fn is_inline_element(tag_name: &str) -> bool {
    matches!(
        tag_name.to_lowercase().as_str(),
        "span" | "a" | "strong" | "em" | "i" | "b" | "u" | "small" | "sub" | "sup"
    )
}

fn get_depth_color(depth: usize) -> egui::Color32 {
    // Generate a color based on the depth, cycling through a palette
    let colors = [
        egui::Color32::from_rgb(255, 100, 100), // Bright red
        egui::Color32::from_rgb(100, 255, 100), // Bright green
        egui::Color32::from_rgb(100, 150, 255), // Bright blue
        egui::Color32::from_rgb(255, 200, 100), // Orange
        egui::Color32::from_rgb(100, 255, 255), // Cyan
        egui::Color32::from_rgb(200, 100, 255), // Purple
    ];
    colors[depth % colors.len()]
}
