use std::sync::{Arc, Mutex};

use crate::{
    api::tabs::BrowserTab,
    html::renderer::{HtmlRenderer, RendererDebugMode},
};

/// Renders the developer tools panel in the browser UI, allowing users to inspect network responses and control the HTML renderer's debug mode.
///
/// # Arguments
/// * `ctx` - The Egui context used for rendering the UI.
/// * `tab` - A mutable reference to the current browser tab, which contains the web client and its responses.
/// * `renderer` - An Arc<Mutex<HtmlRenderer>> that provides access to the HTML renderer, allowing users to change its debug mode.
pub fn render_devtools(
    ctx: &egui::Context,
    tab: &mut BrowserTab,
    renderer: &Arc<Mutex<HtmlRenderer>>,
) {
    egui::SidePanel::right("devtools")
        .exact_width(400.0)
        .frame(
            egui::Frame::new()
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(200, 200, 200),
                ))
                .shadow(egui::epaint::Shadow {
                    spread: 0,
                    offset: [0, -2],
                    blur: 5,
                    color: egui::Color32::from_black_alpha(50),
                })
                .fill(egui::Color32::from_rgb(240, 240, 240))
                .inner_margin(egui::Margin::same(10)),
        )
        .show(ctx, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new("Devtools").heading());
            ui.separator();

            // Renderer Debug Mode Controls section
            ui.label(
                egui::RichText::new("Renderer Debug Mode")
                    .monospace()
                    .strong(),
            );

            let mut renderer_guard = renderer.lock().unwrap();
            let current_mode = renderer_guard.get_debug_mode();

            ui.horizontal(|ui| {
                if ui
                    .selectable_label(current_mode == RendererDebugMode::None, "None")
                    .clicked()
                {
                    renderer_guard.set_debug_mode(RendererDebugMode::None);
                }
                if ui
                    .selectable_label(current_mode == RendererDebugMode::Colors, "Colors")
                    .clicked()
                {
                    renderer_guard.set_debug_mode(RendererDebugMode::Colors);
                }
                if ui
                    .selectable_label(
                        current_mode == RendererDebugMode::ElementText,
                        "Element Text",
                    )
                    .clicked()
                {
                    renderer_guard.set_debug_mode(RendererDebugMode::ElementText);
                }
                if ui
                    .selectable_label(current_mode == RendererDebugMode::Full, "Full")
                    .clicked()
                {
                    renderer_guard.set_debug_mode(RendererDebugMode::Full);
                }
            });

            drop(renderer_guard);

            ui.separator();

            // Network Responses section
            ui.label(
                egui::RichText::new("Network Responses")
                    .monospace()
                    .strong(),
            );
            egui::ScrollArea::both().max_height(300.0).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical(|ui| {
                    for response in &tab.web_client.lock().unwrap().responses {
                        let status_code_color =
                            if response.status_code >= 200 && response.status_code < 300 {
                                egui::Color32::from_rgb(0, 128, 0) // Green for success
                            } else if response.status_code >= 400 {
                                egui::Color32::from_rgb(255, 0, 0) // Red for errors
                            } else {
                                egui::Color32::from_rgb(255, 165, 0) // Orange for other statuses
                            };
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(response.status_code.to_string())
                                    .color(status_code_color)
                                    .monospace(),
                            );
                            ui.separator();
                            ui.label(egui::RichText::new(response.method.clone()).monospace());
                            ui.separator();
                            ui.label(egui::RichText::new(response.url.clone()).monospace());
                        });
                    }
                });
            })
        });
}
