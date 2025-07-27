use iced::{
    Color, Font,
    font::{Style, Weight},
    widget::{Text, text},
};

use crate::util::font::MONOSPACE;

/// Get styling for different HTML elements
pub fn get_text_style_for_element(tag_name: &str, content: String) -> Text<'static, iced::Theme> {
    let base_text = text(content).color(Color::BLACK);

    match tag_name {
        "h1" => base_text.size(32),
        "h2" => base_text.size(28),
        "h3" => base_text.size(24),
        "h4" => base_text.size(20),
        "h5" => base_text.size(18),
        "h6" => base_text.size(16),
        "strong" | "b" => base_text.font(Font {
            weight: Weight::Bold,
            ..Default::default()
        }),
        "em" | "i" => base_text.font(Font {
            style: Style::Italic,
            ..Default::default()
        }),
        "code" => base_text.font(MONOSPACE),
        "small" => base_text.size(12),
        "pre" => base_text.font(MONOSPACE),
        _ => base_text,
    }
}

/// Get margin for different HTML elements
pub fn get_margin_for_element(tag_name: &str) -> u16 {
    match tag_name {
        "p" => 8,
        "h1" => 16,
        "h2" => 14,
        "h3" => 12,
        "h4" => 10,
        "h5" => 8,
        "h6" => 6,
        "pre" => 8,
        "ul" | "ol" => 8,
        "li" => 4,
        _ => 0,
    }
}
