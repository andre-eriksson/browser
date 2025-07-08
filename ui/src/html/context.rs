/// Starts a vertical context for rendering HTML content.
///
/// # Arguments
/// * `ui` - The Egui UI context to render into.
/// * `color` - Optional background color for the context.
/// * `margin` - Optional margin around the context.
/// * `add_contents` - A closure that takes a mutable reference to the Egui UI context and adds contents to it.
pub fn start_vertical_context(
    ui: &mut egui::Ui,
    color: Option<egui::Color32>,
    padding: Option<egui::Margin>,
    margin: Option<egui::Margin>,
    stroke: Option<egui::Stroke>,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    start_context(
        ui,
        color.unwrap_or(egui::Color32::from_rgb(255, 255, 255)),
        padding.unwrap_or_default(),
        margin.unwrap_or_default(),
        stroke,
        |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.vertical(add_contents);
        },
    );
}

/// Starts a horizontal context for rendering HTML content.
///
/// # Arguments
/// * `ui` - The Egui UI context to render into.
/// * `color` - Optional background color for the context.
/// * `margin` - Optional margin around the context.
/// * `is_block` - If true, the context will take the full width available.
/// * `add_contents` - A closure that takes a mutable reference to the Egui UI context and adds contents to it.
pub fn start_horizontal_context(
    ui: &mut egui::Ui,
    color: Option<egui::Color32>,
    padding: Option<egui::Margin>,
    margin: Option<egui::Margin>,
    stroke: Option<egui::Stroke>,
    is_block: bool,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    start_context(
        ui,
        color.unwrap_or(egui::Color32::from_rgb(255, 255, 255)),
        padding.unwrap_or_default(),
        margin.unwrap_or_default(),
        stroke,
        |ui| {
            if is_block {
                ui.set_width(ui.available_width());
            }
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.horizontal(add_contents);
        },
    );
}

fn start_context(
    ui: &mut egui::Ui,
    color: egui::Color32,
    padding: egui::Margin,
    margin: egui::Margin,
    stroke: Option<egui::Stroke>,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    if stroke.is_some() {
        egui::Frame::new()
            .fill(color)
            .inner_margin(padding)
            .outer_margin(margin)
            .stroke(stroke.unwrap())
            .show(ui, add_contents);
    } else {
        egui::Frame::new()
            .fill(color)
            .inner_margin(padding)
            .outer_margin(margin)
            .show(ui, add_contents);
    }
}
