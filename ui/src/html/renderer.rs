use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use api::dom::{ConcurrentDomNode, ConcurrentElement};

use crate::{
    api::tabs::{BrowserTab, TabMetadata},
    html::{
        context::{start_horizontal_context, start_vertical_context},
        inline::InlineRenderer,
        layout::{
            ElementType, get_color_for_element, get_element_type, get_margin_for_element,
            get_padding_for_element, get_stroke_for_element,
        },
        text::get_text_style,
        util::get_color,
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
    debug_mode: RendererDebugMode,

    pub inline_renderer: InlineRenderer,
}

impl HtmlRenderer {
    /// Creates a new instance of the HTML renderer with specified maximum depth and debug mode.
    ///
    /// # Arguments
    /// * `max_depth` - The maximum depth of the HTML document to render, used to prevent excessive recursion.
    /// * `debug_mode` - The debug mode for the renderer, which controls how much information is displayed during rendering.
    pub fn new(debug_mode: RendererDebugMode) -> Self {
        HtmlRenderer {
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
                self.process_dom_children(ui, metadata, tab, &element.children, None);
            });
        }
    }

    pub fn process_dom_children(
        &mut self,
        ui: &mut egui::Ui,
        metadata: &TabMetadata,
        tab: &mut BrowserTab,
        nodes: &[Arc<Mutex<ConcurrentDomNode>>],
        parent_element: Option<&ConcurrentElement>,
    ) {
        for node_arc in nodes {
            let node = node_arc.lock().unwrap().clone();

            match node {
                ConcurrentDomNode::Element(element) => {
                    let has_text_nodes = element
                        .children
                        .iter()
                        .any(|c| matches!(c.lock().unwrap().clone(), ConcurrentDomNode::Text(_)));
                    let has_inline_elements = element
                        .children
                        .iter()
                        .any(|c| matches!(c.lock().unwrap().clone(), ConcurrentDomNode::Element(e) if get_element_type(&e.tag_name) == ElementType::Inline));
                    let is_preformatted = &element.tag_name == "pre";

                    let is_mixed_content = has_inline_elements && has_text_nodes;

                    let color = if self.debug_mode == RendererDebugMode::Full
                        || self.debug_mode == RendererDebugMode::Colors
                    {
                        Some(get_color(&element))
                    } else {
                        Some(get_color_for_element(&element.tag_name))
                    };

                    if &element.tag_name == "hr" {
                        ui.separator();
                        continue;
                    }

                    let margin = Some(get_margin_for_element(&element.tag_name));
                    let padding = Some(get_padding_for_element(&element.tag_name));
                    let stroke = Some(get_stroke_for_element(&element.tag_name));

                    match get_element_type(&element.tag_name) {
                        ElementType::Block => {
                            self.inline_renderer.render(ui, tab, Some(&element));

                            if has_text_nodes && !is_preformatted {
                                start_horizontal_context(
                                    ui,
                                    color,
                                    padding,
                                    margin,
                                    stroke,
                                    true,
                                    |ui| {
                                        self.process_dom_children(
                                            ui,
                                            metadata,
                                            tab,
                                            &element.children,
                                            Some(&element),
                                        );
                                    },
                                );
                                continue;
                            }

                            // NOTE: Might fail for irregular content
                            start_vertical_context(ui, color, padding, margin, stroke, |ui| {
                                if is_preformatted {
                                    self.render_preformatted_elements(ui, tab, &element);
                                } else {
                                    self.process_dom_children(
                                        ui,
                                        metadata,
                                        tab,
                                        &element.children,
                                        Some(&element),
                                    );
                                }
                            });
                        }

                        ElementType::Inline => {
                            self.inline_renderer.collect_element(&element);
                        }

                        ElementType::ListItem => {
                            self.inline_renderer.render(ui, tab, Some(&element));

                            start_horizontal_context(
                                ui,
                                color,
                                padding,
                                margin,
                                stroke,
                                true,
                                |ui| {
                                    if &element.tag_name == "li" {
                                        ui.add(egui::Label::new(" • ").selectable(false));
                                    } else if &element.tag_name == "summary" {
                                        ui.add(egui::Label::new(" ▶ ").selectable(false));
                                    }

                                    self.process_dom_children(
                                        ui,
                                        metadata,
                                        tab,
                                        &element.children,
                                        Some(&element),
                                    );
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
                                        &element.tag_name
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
                                        &element.tag_name
                                    ))
                                    .color(egui::Color32::from_rgb(255, 0, 0)),
                                );
                            }

                            self.process_dom_children(
                                ui,
                                metadata,
                                tab,
                                &element.children,
                                Some(&element),
                            );
                        }
                    }

                    if is_mixed_content {
                        self.inline_renderer.render(ui, tab, Some(&element));
                    }
                }

                ConcurrentDomNode::Text(text) => {
                    if let Some(parent) = parent_element {
                        if get_element_type(&parent.tag_name) != ElementType::Inline {
                            self.inline_renderer.render(ui, tab, Some(&parent));
                        }

                        let styled_text = get_text_style(&parent.tag_name, text.as_str());

                        ui.label(styled_text);
                    } else {
                        ui.label(text);
                    }
                }

                _ => {}
            }
        }

        self.inline_renderer.render(ui, tab, parent_element);
    }

    fn render_preformatted_elements(
        &mut self,
        ui: &mut egui::Ui,
        tab: &mut BrowserTab,
        element: &ConcurrentElement,
    ) {
        let mut children: VecDeque<Arc<Mutex<ConcurrentDomNode>>> =
            element.children.iter().cloned().collect();
        let mut new_context = true;
        let mut new_lines: u8 = 0;

        while children.front().is_some() {
            if new_context {
                if new_lines == 2 {
                    ui.label("");
                    new_lines = 0;
                }
                start_horizontal_context(ui, None, None, None, None, false, |ui| {
                    self.render_children_with_context(
                        ui,
                        tab,
                        element,
                        &mut new_context,
                        &mut new_lines,
                        &mut children,
                    );
                });
            }
        }
    }

    fn render_children_with_context(
        &mut self,
        ui: &mut egui::Ui,
        tab: &mut BrowserTab,
        element: &ConcurrentElement,
        new_context: &mut bool,
        new_lines: &mut u8,
        children: &mut VecDeque<Arc<Mutex<ConcurrentDomNode>>>,
    ) {
        while let Some(child) = children.pop_front() {
            match child.lock().unwrap().clone() {
                ConcurrentDomNode::Text(text) => {
                    if text == "\r\n" || text == "\n" {
                        *new_lines += 1;
                        *new_context = true;
                        break;
                    }
                    *new_lines = 0;
                    let styled_text = get_text_style(
                        &element.tag_name,
                        text.replace("\r\n", "").replace('\n', "").as_str(),
                    );
                    ui.label(styled_text.monospace());
                    if text.ends_with("\r\n") || text.ends_with('\n') {
                        *new_context = true;
                        break;
                    }
                }
                ConcurrentDomNode::Element(child_element) => {
                    match get_element_type(&child_element.tag_name) {
                        ElementType::Block => {
                            if *new_context {
                                *new_context = false;
                                *new_lines = 0;
                            }
                            self.inline_renderer.render(ui, tab, Some(&child_element));
                            start_vertical_context(
                                ui,
                                None,
                                Some(get_padding_for_element(&child_element.tag_name)),
                                Some(get_margin_for_element(&child_element.tag_name)),
                                Some(get_stroke_for_element(&child_element.tag_name)),
                                |ui| {
                                    self.process_dom_children(
                                        ui,
                                        &TabMetadata::default(), // Placeholder for metadata
                                        tab,
                                        &child_element.children,
                                        Some(&child_element),
                                    );
                                },
                            );
                        }
                        ElementType::Inline => {
                            self.inline_renderer.collect_element(&child_element);
                            self.inline_renderer.render(ui, tab, Some(&element));
                        }
                        _ => (),
                    }
                }
                _ => {}
            }
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
