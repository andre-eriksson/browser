/// Renders text with appropriate styles based on HTML tags.
///
/// # Arguments
/// * `tag_name` - The name of the HTML tag (e.g., "h1", "p", "strong").
/// * `text` - The text content to be styled.
///
/// # Returns
/// * `egui::RichText` - The styled text ready for rendering in Egui.
pub fn get_text_style(tag_name: &str, text: &str) -> egui::RichText {
    match tag_name {
        "h1" => egui::RichText::new(text).strong().size(32.0),
        "h2" => egui::RichText::new(text).strong().size(24.0),
        "h3" => egui::RichText::new(text).strong().size(20.0),
        "h4" => egui::RichText::new(text).strong(),
        "h5" => egui::RichText::new(text).strong().size(10.0),
        "h6" => egui::RichText::new(text).strong().size(8.0),
        "strong" | "b" | "th" => egui::RichText::new(text).strong(),
        "em" | "i" | "address" => egui::RichText::new(text).italics(),
        "u" => egui::RichText::new(text).underline(),
        "s" | "strike" => egui::RichText::new(text).strikethrough(),
        "del" => egui::RichText::new(text)
            .background_color(egui::Color32::from_rgb(255, 187, 187))
            .strikethrough(),
        "ins" => egui::RichText::new(text).background_color(egui::Color32::from_rgb(212, 252, 188)),
        "code" | "pre" => {
            // Code blocks and inline code
            egui::RichText::new(text).monospace()
        }
        "a" => egui::RichText::new(text)
            .color(egui::Color32::from_rgb(0, 0, 255))
            .underline(),
        _ => {
            // Default text style for other tags
            egui::RichText::new(text)
        }
    }
}
