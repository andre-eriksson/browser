use api::dom::{ConcurrentDomNode, ConcurrentElement};

use crate::{
    api::tabs::BrowserTab,
    html::{
        context::start_horizontal_context, image::render_image, renderer::RendererDebugMode,
        text::get_text_style, util::resolve_path,
    },
};

/// InlineRenderer is responsible for rendering inline HTML elements such as text and images.
///
/// # Fields
/// * `buffer` - A vector that collects inline elements to be rendered.
/// * `debug_mode` - The debug mode for the renderer, which controls how much information is displayed during rendering.
#[derive(Debug, Clone)]
pub struct InlineRenderer {
    buffer: Vec<ConcurrentElement>,
    debug_mode: RendererDebugMode,
}

impl InlineRenderer {
    /// Creates a new InlineRenderer instance with the specified debug mode.
    ///
    /// # Arguments
    /// * `debug_mode` - The debug mode for the renderer, which controls how much information is displayed during rendering.
    pub fn new(debug_mode: RendererDebugMode) -> Self {
        InlineRenderer {
            buffer: Vec::new(),
            debug_mode,
        }
    }

    /// Collects an inline HTML element to be rendered later.
    ///
    /// # Arguments
    /// * `element` - A reference to the ConcurrentElement to be collected.
    pub fn collect_element(&mut self, element: &ConcurrentElement) {
        self.buffer.push(element.clone());
    }

    /// Renders the collected inline elements into the provided UI context.
    ///
    /// # Arguments
    /// * `ui` - The Egui UI context where the elements will be rendered.
    /// * `tab` - A mutable reference to the BrowserTab, which may be used for navigation or other tab-related actions.
    pub fn render(&mut self, ui: &mut egui::Ui, tab: &mut BrowserTab) {
        if self.buffer.is_empty() {
            return; // Nothing to render
        }

        let color = if self.debug_mode == RendererDebugMode::Full
            || self.debug_mode == RendererDebugMode::Colors
        {
            Some(egui::Color32::from_rgb(240, 240, 240))
        } else {
            None
        };

        start_horizontal_context(ui, color, None, false, |ui| {
            for element in &self.buffer {
                self.render_element(ui, tab, element);
            }
        });

        self.buffer.clear(); // Clear the buffer after rendering
    }

    fn render_element(&self, ui: &mut egui::Ui, tab: &mut BrowserTab, element: &ConcurrentElement) {
        if element.children.is_empty() {
            if element.tag_name.as_str() == "img" {
                render_image(ui, element, &tab.url);
            }
        }

        for child in &element.children {
            match child.lock().unwrap().clone() {
                ConcurrentDomNode::Element(child_element) => {
                    match child_element.tag_name.as_str() {
                        "img" => {
                            render_image(ui, &child_element, &tab.url);
                        }
                        "script" | "style" => {
                            // Skip script and style elements
                            if self.debug_mode == RendererDebugMode::Full
                                || self.debug_mode == RendererDebugMode::ElementText
                            {
                                ui.label(format!("Skipping element: <{}>", child_element.tag_name));
                            }
                        }
                        _ => {
                            self.render_element(ui, tab, &child_element);
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
                            href.clone()
                        } else {
                            text.clone()
                        };

                        let long_href = resolve_path(&tab.url, &href);

                        let element = get_text_style(&element.tag_name, &link_text);
                        let response =
                            ui.add(egui::Label::new(element).sense(egui::Sense::click()));

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
                            tab.navigate_to(long_href);
                        }
                    }
                    _ => {
                        let element = get_text_style(&element.tag_name, &text);

                        ui.label(element);
                    }
                },
                _ => {}
            }
        }
    }
}
