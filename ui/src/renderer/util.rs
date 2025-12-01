use html_syntax::{HtmlTag, KnownTag};
use iced::{
    Color, Font,
    font::{Style, Weight},
    widget::{Text, text},
};

use crate::util::font::MONOSPACE;

/// Get styling for different HTML elements
pub fn get_text_style_for_element(tag: &HtmlTag, content: String) -> Text<'static, iced::Theme> {
    let base_text = text(content).color(Color::BLACK);

    match tag {
        HtmlTag::Known(KnownTag::H1) => base_text.size(32),
        HtmlTag::Known(KnownTag::H2) => base_text.size(28),
        HtmlTag::Known(KnownTag::H3) => base_text.size(24),
        HtmlTag::Known(KnownTag::H4) => base_text.size(20),
        HtmlTag::Known(KnownTag::H5) => base_text.size(18),
        HtmlTag::Known(KnownTag::H6) => base_text.size(16),
        HtmlTag::Known(KnownTag::Strong) | HtmlTag::Known(KnownTag::B) => base_text.font(Font {
            weight: Weight::Bold,
            ..Default::default()
        }),
        HtmlTag::Known(KnownTag::Em) | HtmlTag::Known(KnownTag::I) => base_text.font(Font {
            style: Style::Italic,
            ..Default::default()
        }),
        HtmlTag::Known(KnownTag::Code) => base_text.font(MONOSPACE),
        HtmlTag::Known(KnownTag::Small) => base_text.size(12),
        HtmlTag::Known(KnownTag::Pre) => base_text.font(MONOSPACE),
        _ => base_text,
    }
}

/// Get margin for different HTML elements
pub fn get_margin_for_element(tag: &HtmlTag) -> u16 {
    match tag {
        HtmlTag::Known(KnownTag::P) => 8,
        HtmlTag::Known(KnownTag::H1) => 16,
        HtmlTag::Known(KnownTag::H2) => 14,
        HtmlTag::Known(KnownTag::H3) => 12,
        HtmlTag::Known(KnownTag::H4) => 10,
        HtmlTag::Known(KnownTag::H5) => 8,
        HtmlTag::Known(KnownTag::H6) => 6,
        HtmlTag::Known(KnownTag::Pre) => 8,
        HtmlTag::Known(KnownTag::Ul) | HtmlTag::Known(KnownTag::Ol) => 8,
        HtmlTag::Known(KnownTag::Li) => 4,
        _ => 0,
    }
}
