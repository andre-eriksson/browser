use api::dom::{ConcurrentDomNode, ConcurrentElement};
use tracing::warn;

use crate::{
    api::tabs::{BrowserTab, TabMetadata},
    html::util::{get_depth_color, is_inline_element, resolve_path},
};

#[derive(Debug, PartialEq, Eq)]
pub enum RendererDebugMode {
    /// Renders all elements with color backgrounds and text labels for debugging.
    /// This mode is useful for visualizing the structure of HTML elements.
    Full,

    /// Renders the structure of HTML elements with colors, but without text labels.
    Colors,

    /// Renders only the text content of HTML elements without any additional styling.
    ElementText,

    /// Disables all debugging features, rendering elements as they would normally appear.
    None,
}

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
    inline_buffer: Vec<ConcurrentElement>,
    debug: RendererDebugMode,
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        HtmlRenderer {
            max_depth: 100,
            current_depth: 0,
            inline_buffer: Vec::new(),
            debug: RendererDebugMode::None,
        }
    }
}

impl HtmlRenderer {
    /// Creates a new `HtmlRenderer` with the specified maximum depth and debug mode.
    ///
    /// # Arguments
    /// * `max_depth` - The maximum depth to render HTML elements.
    /// * `debug` - If true, enables debug mode which displays additional information about the rendering process.
    pub fn new(max_depth: usize, debug: RendererDebugMode) -> Self {
        HtmlRenderer {
            max_depth,
            current_depth: 0,
            inline_buffer: Vec::new(),
            debug,
        }
    }

    /// Displays the HTML content of a tab
    pub fn display(
        &mut self,
        ui: &mut egui::Ui,
        metadata: &TabMetadata,
        element: &ConcurrentElement,
        tab: &mut BrowserTab,
    ) {
        self.current_depth = 0; // Reset depth for each new element
        self.inline_buffer.clear(); // Clear inline buffer for each new element
        self.display_body(ui, metadata, element, tab);
    }

    fn display_body(
        &mut self,
        ui: &mut egui::Ui,
        metadata: &TabMetadata,
        element: &ConcurrentElement,
        tab: &mut BrowserTab,
    ) {
        self.initialize_block_context(ui, metadata, element, tab);
    }

    fn display_element(
        &mut self,
        ui: &mut egui::Ui,
        metadata: &TabMetadata,
        element: &ConcurrentElement,
        tab: &mut BrowserTab,
    ) {
        if self.current_depth > self.max_depth {
            ui.label(format!("{}... (depth limit reached)", element.tag_name));
            return;
        }

        self.current_depth += 1;

        match element.tag_name.as_str() {
            "div" | "header" | "footer" | "main" | "section" | "article" | "aside" | "pre"
            | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "hr" => {
                if !self.inline_buffer.is_empty() {
                    self.render_inline_elements(ui, tab);
                }

                self.initialize_block_context(ui, metadata, element, tab);
            }
            "span" | "a" | "strong" | "em" | "i" | "b" | "u" | "code" | "small" | "sub" | "sup"
            | "img" => {
                self.collect_inline_elements(ui, element);
            }
            "script" | "style" => {
                // Skip script and style tags in the rendering
                if self.debug == RendererDebugMode::Full
                    || self.debug == RendererDebugMode::ElementText
                {
                    ui.label(format!(
                        "Skipping: <{}> (depth: {})",
                        element.tag_name, self.current_depth
                    ));
                }
            }
            _ => {
                // Handle unrecognized elements
                if self.debug == RendererDebugMode::Full
                    || self.debug == RendererDebugMode::ElementText
                {
                    ui.label(format!(
                        "E: <{}> (depth: {})",
                        element.tag_name, self.current_depth
                    ));
                }

                // Process children of unrecognized elements
                for child in &element.children {
                    if !is_inline_element(&element.tag_name) {
                        // If the current element is not an inline element, render any collected inline elements
                        if !self.inline_buffer.is_empty() {
                            self.render_inline_elements(ui, tab);
                        }
                    }

                    match child.lock().unwrap().clone() {
                        ConcurrentDomNode::Element(child_element) => {
                            self.display_element(ui, metadata, &child_element, tab);
                        }
                        ConcurrentDomNode::Text(text) => {
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
        element: &ConcurrentElement,
        tab: &mut BrowserTab,
    ) {
        if self.current_depth > self.max_depth {
            ui.label(format!("{}... (depth limit reached)", element.tag_name));
            return;
        }

        let color = match self.debug {
            RendererDebugMode::Full | RendererDebugMode::Colors => {
                get_depth_color(self.current_depth)
            } // Use a color based on the current depth for debug mode
            _ => egui::Color32::from_rgb(255, 255, 255), // Default white for normal mode
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
            "div" => egui::Margin::same(0),
            _ => {
                // Base margin for other block elements
                egui::Margin::symmetric(0, 4)
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
                let has_text_nodes = element.children.iter().any(|child| {
                    matches!(child.lock().unwrap().clone(), ConcurrentDomNode::Text(_))
                });

                if element.tag_name == "body" || !has_text_nodes {
                    ui.vertical(|ui| {
                        self.render_block_element(ui, metadata, element, tab);
                    });
                } else {
                    ui.horizontal(|ui| {
                        self.render_block_element(ui, metadata, element, tab);
                    });
                }
            });
    }

    fn render_block_element(
        &mut self,
        ui: &mut egui::Ui,
        metadata: &TabMetadata,
        element: &ConcurrentElement,
        tab: &mut BrowserTab,
    ) {
        if self.current_depth > self.max_depth {
            ui.label(format!("{}... (depth limit reached)", element.tag_name));
            return;
        }

        if self.debug == RendererDebugMode::Full || self.debug == RendererDebugMode::ElementText {
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

        let has_inline_elements = element.children.iter().any(|child| {
            matches!(child.lock().unwrap().clone(), ConcurrentDomNode::Element(e) if is_inline_element(&e.tag_name))
        });

        let has_text_nodes = element
            .children
            .iter()
            .any(|child| matches!(child.lock().unwrap().clone(), ConcurrentDomNode::Text(_)));

        let is_mixed_content = has_inline_elements && has_text_nodes;

        // Recursively display child elements
        for child in &element.children {
            match child.lock().unwrap().clone() {
                ConcurrentDomNode::Element(child_element) => {
                    if is_inline_element(&child_element.tag_name) {
                        self.collect_inline_elements(ui, &child_element);
                    } else {
                        if !self.inline_buffer.is_empty() {
                            self.render_inline_elements(ui, tab);
                        }

                        self.display_element(ui, metadata, &child_element, tab);
                    }
                    if is_mixed_content && !self.inline_buffer.is_empty() {
                        self.render_inline_elements(ui, tab);
                    }
                }
                ConcurrentDomNode::Text(text) => {
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
            self.render_inline_elements(ui, tab);
        }
    }

    fn collect_inline_elements(&mut self, ui: &mut egui::Ui, element: &ConcurrentElement) {
        if self.current_depth > self.max_depth {
            ui.label(format!("{}... (depth limit reached)", element.tag_name));
            return;
        }

        // Collect inline elements into a buffer and render them later due to egui limitations in rendering
        self.inline_buffer.push(element.clone());
    }

    fn render_inline_elements(&mut self, ui: &mut egui::Ui, tab: &mut BrowserTab) {
        if self.inline_buffer.is_empty() {
            return;
        }

        let color =
            if self.debug == RendererDebugMode::Full || self.debug == RendererDebugMode::Colors {
                egui::Color32::from_rgb(240, 240, 240) // Light gray for debug mode
            } else {
                egui::Color32::from_rgb(255, 255, 255) // White for normal mode
            };

        egui::Frame::new().fill(color).show(ui, |ui| {
            ui.horizontal(|ui| {
                for inline_element in &self.inline_buffer {
                    self.render_inline_element(ui, inline_element, tab);
                }
            });
        });

        // Clear the buffer after rendering
        self.inline_buffer.clear();
    }

    fn render_inline_element(
        &self,
        ui: &mut egui::Ui,
        element: &ConcurrentElement,
        tab: &mut BrowserTab,
    ) {
        // Handle self-closing or empty inline elements
        if element.children.is_empty() {
            match element.tag_name.as_str() {
                "img" => {
                    self.render_image(ui, element, tab.url.as_str());
                }
                _ => {
                    // For other empty inline elements, you might want to render some placeholder or skip
                }
            }
            return;
        }

        for child in &element.children {
            match child.lock().unwrap().clone() {
                ConcurrentDomNode::Element(child_element) => {
                    match child_element.tag_name.as_str() {
                        "img" => {
                            self.render_image(ui, &child_element, tab.url.as_str());
                        }
                        "script" | "style" => {
                            // Skip script and style tags in the rendering
                            if self.debug == RendererDebugMode::Full
                                || self.debug == RendererDebugMode::ElementText
                            {
                                ui.label(format!(
                                    "Skipping: <{}> (depth: {})",
                                    child_element.tag_name, self.current_depth
                                ));
                            }
                        }
                        _ => {
                            self.render_inline_element(ui, &child_element, tab);
                        }
                    }
                }
                ConcurrentDomNode::Text(text) => match element.tag_name.as_str() {
                    "code" | "pre" => {
                        ui.label(egui::RichText::new(text).monospace());
                    }
                    "a" => {
                        let href = element.attributes.get("href").cloned().unwrap_or_default();
                        let link_text = if text.is_empty() {
                            "Link".to_string()
                        } else {
                            text.clone()
                        };

                        let long_href = resolve_path(&tab.url, &href);

                        let response = ui.add(
                            egui::Label::new(
                                egui::RichText::new(link_text)
                                    .color(egui::Color32::from_rgb(0, 0, 255))
                                    .underline(),
                            )
                            .sense(egui::Sense::click()),
                        );

                        let clicked = response.clicked();

                        if response.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            response.on_hover_ui(|ui| {
                                ui.label(
                                    egui::RichText::new(long_href.clone())
                                        .color(egui::Color32::BLACK),
                                );
                            });
                        }

                        if clicked {
                            // Handle link click
                            tab.navigate_to(long_href);
                        }
                    }

                    _ => {
                        ui.label(text);
                    }
                },
                _ => {}
            }
        }
    }

    fn render_image(&self, ui: &mut egui::Ui, element: &ConcurrentElement, url: &str) {
        let src = element.attributes.get("src");

        if src.is_none() {
            warn!("Image element <{}> missing 'src' attribute", element.id);
            return;
        }

        let src = src.unwrap();

        if src.contains(".svg") {
            // TODO: Handle SVG images.
            return;
        }

        let image_url = resolve_path(url, src);

        let width = element
            .attributes
            .get("width")
            .and_then(|w| w.parse::<f32>().ok());
        let height = element
            .attributes
            .get("height")
            .and_then(|h| h.parse::<f32>().ok());
        let alt = element.attributes.get("alt").cloned().unwrap_or_default();

        // Add some spacing between images to act like text spacing
        ui.spacing_mut().item_spacing.x = 4.0;
        let mut image = egui::Image::new(image_url);

        image = image.alt_text(alt.clone());

        image = image.fit_to_exact_size(egui::Vec2::new(
            width.unwrap_or(100.0),  // Default width if not specified
            height.unwrap_or(100.0), // Default height if not specified
        ));

        ui.add(image).on_hover_ui(|ui| {
            ui.label(egui::RichText::new(alt).color(egui::Color32::BLACK));
        });
    }
}
