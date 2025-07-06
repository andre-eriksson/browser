use api::dom::{ConcurrentDomNode, ConcurrentElement};

use crate::{
    api::tabs::{BrowserTab, TabMetadata},
    html::{
        context::{start_horizontal_context, start_vertical_context},
        inline::InlineRenderer,
        layout::{
            ElementType, get_element_type, get_margin_for_element, get_padding_for_element,
            get_stroke_for_element,
        },
        text::get_text_style,
        util::get_depth_color,
    },
};

/// Represents the debug mode for the HTML renderer, useful for checking that margins, padding, and other layout properties are applied correctly during rendering.
///
/// # Variants
/// * `Full` - Displays all debug information, including element types and text content.
/// * `Colors` - Displays debug information with colors based on element depth.
/// * `ElementText` - Displays only the text content of elements, useful for debugging text rendering.
/// * `None` - No debug information is displayed, used for normal rendering without additional output.
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RendererDebugMode {
    Full,
    Colors,
    ElementText,
    None,
}

/// Represents an HTML renderer that processes and displays HTML content in a structured way.
/// It handles rendering of HTML elements, including inline and block elements, while respecting the maximum depth to prevent excessive recursion.
///
/// # Fields
/// * `max_depth` - The maximum depth of the HTML document to render, used to prevent excessive recursion.
/// * `current_depth` - The current depth of the rendering process, used to track how deep into the HTML structure the renderer is.
/// * `debug_mode` - The debug mode for the renderer, which controls how much information is displayed during rendering.
/// * `inline_renderer` - An instance of `InlineRenderer` that collects and renders inline elements separately from block elements.
#[derive(Debug, Clone)]
pub struct HtmlRenderer {
    max_depth: usize,
    current_depth: usize,
    debug_mode: RendererDebugMode,

    pub inline_renderer: InlineRenderer,
}

impl HtmlRenderer {
    /// Creates a new instance of the HTML renderer with specified maximum depth and debug mode.
    ///
    /// # Arguments
    /// * `max_depth` - The maximum depth of the HTML document to render, used to prevent excessive recursion.
    /// * `debug_mode` - The debug mode for the renderer, which controls how much information is displayed during rendering.
    pub fn new(max_depth: usize, debug_mode: RendererDebugMode) -> Self {
        HtmlRenderer {
            max_depth,
            current_depth: 0,
            debug_mode,
            inline_renderer: InlineRenderer::new(debug_mode),
        }
    }

    /// Starts rendering the HTML content for a given element, must contain a body tag.
    ///
    /// # Arguments
    /// * `ui` - The Egui UI context to render the HTML content into.
    /// * `metadata` - Metadata about the current tab, such as URL and title.
    /// * `tab` - The current browser tab being rendered.
    /// * `element` - The root HTML element to start rendering from, typically the `<body>` tag.
    pub fn display(
        &mut self,
        ui: &mut egui::Ui,
        metadata: &TabMetadata,
        tab: &mut BrowserTab,
        element: &ConcurrentElement,
    ) {
        if self.current_depth >= self.max_depth {
            ui.label(format!("{}... (depth limit reached)", element.tag_name));
            return;
        }

        let color = if self.debug_mode == RendererDebugMode::Full
            || self.debug_mode == RendererDebugMode::Colors
        {
            Some(egui::Color32::from_rgb(240, 240, 240))
        } else {
            None
        };

        let margin = Some(get_margin_for_element(&element.tag_name));
        let padding = Some(get_padding_for_element(&element.tag_name));

        if element.tag_name == "body" {
            start_vertical_context(ui, color, padding, margin, None, |ui| {
                self.process_child_elements(ui, metadata, tab, element);
            });
        }
    }

    fn process_child_elements(
        &mut self,
        ui: &mut egui::Ui,
        metadata: &TabMetadata,
        tab: &mut BrowserTab,
        element: &ConcurrentElement,
    ) {
        // Check depth limit before processing
        if self.current_depth >= self.max_depth {
            ui.label(format!(
                "... (depth limit reached for {})",
                element.tag_name
            ));
            return;
        }

        self.current_depth += 1;

        for child in &element.children {
            match child.lock().unwrap().clone() {
                ConcurrentDomNode::Element(child_element) => {
                    let has_text_nodes = child_element
                        .children
                        .iter()
                        .any(|c| matches!(c.lock().unwrap().clone(), ConcurrentDomNode::Text(_)));
                    let has_inline_elements = child_element
                        .children
                        .iter()
                        .any(|c| matches!(c.lock().unwrap().clone(), ConcurrentDomNode::Element(e) if get_element_type(&e.tag_name) == ElementType::Inline));

                    let is_mixed_content = has_inline_elements && has_text_nodes;

                    let color = if self.debug_mode == RendererDebugMode::Full
                        || self.debug_mode == RendererDebugMode::Colors
                    {
                        Some(get_depth_color(self.current_depth))
                    } else {
                        None
                    };

                    let margin = Some(get_margin_for_element(&child_element.tag_name));
                    let padding = Some(get_padding_for_element(&child_element.tag_name));
                    let stroke = Some(get_stroke_for_element(&child_element.tag_name));

                    match get_element_type(&child_element.tag_name) {
                        ElementType::Block => {
                            // Render any previously collected inline elements before starting a new block
                            self.inline_renderer.render(ui, tab);

                            if has_text_nodes {
                                // If there are text nodes, use horizontal context e.g. <p>
                                start_horizontal_context(
                                    ui,
                                    color,
                                    padding,
                                    margin,
                                    stroke,
                                    true,
                                    |ui| {
                                        self.process_child_elements(
                                            ui,
                                            metadata,
                                            tab,
                                            &child_element,
                                        );
                                    },
                                );
                                continue;
                            }

                            // If there are no text nodes, use vertical context e.g. semantic elements (usually)
                            // NOTE: Might fail for irregular content
                            start_vertical_context(ui, color, padding, margin, stroke, |ui| {
                                self.process_child_elements(ui, metadata, tab, &child_element);
                            });
                        }

                        ElementType::Inline => {
                            self.inline_renderer.collect_element(&child_element);
                        }

                        ElementType::ListItem => {
                            self.inline_renderer.render(ui, tab);

                            start_horizontal_context(
                                ui,
                                color,
                                padding,
                                margin,
                                stroke,
                                true,
                                |ui| {
                                    // Add bullet point
                                    if &child_element.tag_name == "li" {
                                        ui.label(" • ");
                                    } else if &child_element.tag_name == "summary" {
                                        ui.label(" ▶ ");
                                    }

                                    // Process the content of the <li> element
                                    self.process_child_elements(ui, metadata, tab, &child_element);
                                },
                            );
                        }

                        ElementType::Skip => {
                            if self.debug_mode == RendererDebugMode::Full
                                || self.debug_mode == RendererDebugMode::ElementText
                            {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Skipping element: <{}>",
                                        &child_element.tag_name
                                    ))
                                    .color(egui::Color32::from_rgb(255, 165, 0)),
                                );
                            }
                        }

                        ElementType::Unknown => {
                            if self.debug_mode == RendererDebugMode::Full
                                || self.debug_mode == RendererDebugMode::ElementText
                            {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Unknown element: <{}>",
                                        &child_element.tag_name
                                    ))
                                    .color(egui::Color32::from_rgb(255, 0, 0)),
                                );
                            }

                            // If the element is unknown, we can still render its children
                            self.process_child_elements(ui, metadata, tab, &child_element);
                        }
                    }

                    if is_mixed_content {
                        // Render any inline elements collected so far
                        self.inline_renderer.render(ui, tab);
                    }
                }

                ConcurrentDomNode::Text(text) => {
                    if get_element_type(&element.tag_name) != ElementType::Inline {
                        self.inline_renderer.render(ui, tab);
                    }

                    let styled_text = get_text_style(&element.tag_name, &text);

                    ui.label(styled_text);
                }

                _ => continue, // Skip unsupported node types
            }
        }

        // Render any remaining inline elements after processing all children
        self.inline_renderer.render(ui, tab);

        if self.current_depth > 0 {
            // Decrement current depth only if we are not at the root level
            self.current_depth -= 1;
        }
    }

    /// Gets the current debug mode of the renderer.
    ///
    /// # Returns
    /// * The current debug mode of the renderer, which can be used to determine how much information is displayed during rendering.
    pub fn get_debug_mode(&self) -> RendererDebugMode {
        self.debug_mode
    }

    /// Sets the debug mode of the renderer and updates the inline renderer accordingly.
    ///
    /// # Arguments
    /// * `debug_mode` - The new debug mode to set for the renderer.
    pub fn set_debug_mode(&mut self, debug_mode: RendererDebugMode) {
        self.debug_mode = debug_mode;
        self.inline_renderer = InlineRenderer::new(debug_mode);
    }
}
