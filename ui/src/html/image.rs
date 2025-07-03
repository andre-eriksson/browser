use api::dom::ConcurrentElement;
use tracing::warn;

use crate::html::util::resolve_path;

/// Renders an image element in the HTML content.
///
/// # Arguments
/// * `ui` - The Egui UI context to render the image into.
/// * `element` - The HTML element representing the image.
/// * `base_url` - The base URL to resolve relative image paths, both /image and http(s):// URLs work.
pub fn render_image(ui: &mut egui::Ui, element: &ConcurrentElement, base_url: &str) {
    let src = match element.attributes.get("src") {
        Some(src) => src,
        None => {
            warn!("Image element missing 'src' attribute");
            return;
        }
    };

    if src.contains(".svg") {
        // TODO: Handle SVG images
        return;
    }

    let image_url = resolve_path(base_url, src);
    let dimensions = calculate_dimensions(&element);
    let alt_text = element.attributes.get("alt").cloned().unwrap_or_default();

    ui.spacing_mut().item_spacing.x = 4.0;

    let image = egui::Image::new(image_url)
        .alt_text(alt_text.clone())
        .fit_to_exact_size(dimensions);

    ui.add(image).on_hover_ui(|ui| {
        ui.label(alt_text);
    });
}

fn calculate_dimensions(element: &ConcurrentElement) -> egui::Vec2 {
    let width = element
        .attributes
        .get("width")
        .and_then(|w| w.parse::<f32>().ok())
        .unwrap_or(100.0);
    let height = element
        .attributes
        .get("height")
        .and_then(|h| h.parse::<f32>().ok())
        .unwrap_or(100.0);
    egui::Vec2::new(width, height)
}
