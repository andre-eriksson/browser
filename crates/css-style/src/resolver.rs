use crate::types::{
    border::{
        Border, BorderColor, BorderColorValue, BorderStyle, BorderStyleValue, BorderWidth,
        BorderWidthValue,
    },
    color::{Color, NamedColor, SystemColor},
    display::{BoxDisplay, Display, InsideDisplay, OutsideDisplay},
    font::{AbsoluteSize, FontFamily, FontFamilyName, FontSize, GenericName, RelativeSize},
    global::Global,
    height::Height,
    length::{Length, LengthUnit},
    line_height::LineHeight,
    margin::{Margin, MarginValue},
    padding::{Padding, PaddingValue},
    position::Position,
    text_align::TextAlign,
    width::Width,
};

/// A resolver for CSS property values
pub struct PropertyResolver;

impl PropertyResolver {
    pub fn resolve_border(value: &str) -> Option<Border> {
        let mut border = Border::none();

        let parts = value.split_whitespace().collect::<Vec<&str>>();

        // Only supporting: '<length> <style> <color>' for now

        for part in parts {
            if let Some(length) = PropertyResolver::resolve_length(part) {
                border.width = BorderWidth::all(BorderWidthValue::Length(length))
            }

            if let Some(style) = BorderStyleValue::parse(value) {
                border.style = BorderStyle::all(style);
            }

            if let Some(color) = PropertyResolver::resolve_color(part) {
                border.color = BorderColor::all(BorderColorValue::Color(color));
            }
        }

        Some(border)
    }

    pub fn resolve_color(value: &str) -> Option<Color> {
        if value.starts_with('#') {
            return Color::hex(value);
        }

        if let Some(rgb_color) = Color::from_rgb_string(value) {
            return Some(rgb_color);
        }

        let named_color = NamedColor::from(value);
        if named_color != NamedColor::Unknown {
            return Some(Color::Named(named_color));
        }

        let system_color = SystemColor::from(value);
        if system_color != SystemColor::Unknown {
            return Some(Color::System(system_color));
        }

        None
    }

    pub fn resolve_display(value: &str) -> Option<Display> {
        let parts = value.split_whitespace().collect::<Vec<&str>>();

        if parts.len() == 2 {
            let outside = match parts[0] {
                "block" => Some(OutsideDisplay::Block),
                "inline" => Some(OutsideDisplay::Inline),
                _ => None,
            };

            let inside = match parts[1] {
                "flow" => Some(InsideDisplay::Flow),
                "flow-root" => Some(InsideDisplay::FlowRoot),
                "table" => Some(InsideDisplay::Table),
                "flex" => Some(InsideDisplay::Flex),
                "grid" => Some(InsideDisplay::Grid),
                "ruby" => Some(InsideDisplay::Ruby),
                _ => None,
            };

            if outside.is_some() && inside.is_some() {
                return Some(Display {
                    outside,
                    inside,
                    internal: None,
                    box_display: None,
                    global: None,
                });
            }
        } else if parts.len() == 1 {
            if let Some(global_value) = Global::parse(parts[0]) {
                return Some(Display {
                    outside: None,
                    inside: None,
                    internal: None,
                    box_display: None,
                    global: Some(global_value),
                });
            }

            match parts[0] {
                "inline" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Inline),
                        inside: Some(InsideDisplay::Flow),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "inline-block" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Inline),
                        inside: Some(InsideDisplay::FlowRoot),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "inline-table" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Inline),
                        inside: Some(InsideDisplay::Table),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "inline-flex" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Inline),
                        inside: Some(InsideDisplay::Flex),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "inline-grid" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Inline),
                        inside: Some(InsideDisplay::Grid),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "block" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Block),
                        inside: Some(InsideDisplay::Flow),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "flow" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Block),
                        inside: Some(InsideDisplay::Flow),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "flow-root" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Block),
                        inside: Some(InsideDisplay::FlowRoot),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "table" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Block),
                        inside: Some(InsideDisplay::Table),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "flex" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Block),
                        inside: Some(InsideDisplay::Flex),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "grid" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Block),
                        inside: Some(InsideDisplay::Grid),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "ruby" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Inline),
                        inside: Some(InsideDisplay::Ruby),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "contents" => {
                    return Some(Display {
                        outside: None,
                        inside: None,
                        internal: None,
                        box_display: Some(BoxDisplay::Contents),
                        global: None,
                    });
                }
                "none" => {
                    return Some(Display {
                        outside: None,
                        inside: None,
                        internal: None,
                        box_display: Some(BoxDisplay::None),
                        global: None,
                    });
                }
                _ => {}
            }
        }

        None
    }

    pub fn resolve_font_family(value: &str) -> Option<FontFamily> {
        let families = value
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();

        if families.is_empty() {
            return None;
        }

        Some(FontFamily {
            names: families
                .into_iter()
                .map(|name| {
                    if let Some(generic) = GenericName::parse(&name) {
                        FontFamilyName::Generic(generic)
                    } else {
                        let unquoted = name.trim_matches('"').trim_matches('\'').to_string();
                        FontFamilyName::Specific(unquoted)
                    }
                })
                .collect(),
        })
    }

    pub fn resolve_font_size(value: &str) -> Option<FontSize> {
        if let Some(absolute_size) = AbsoluteSize::parse(value) {
            return Some(FontSize::Absolute(absolute_size));
        }

        if let Some(relative_size) = RelativeSize::parse(value) {
            return Some(FontSize::Relative(relative_size));
        }

        if let Some(length) = PropertyResolver::resolve_length(value) {
            return Some(FontSize::Length(length));
        }

        if let Some(percentage) = PropertyResolver::resolve_percentage(value) {
            return Some(FontSize::Percentage(percentage));
        }

        None
    }

    pub fn resolve_height(value: &str) -> Option<Height> {
        let basic = match value {
            "auto" => Some(Height::Auto),
            "max-content" => Some(Height::MaxContent),
            "min-content" => Some(Height::MinContent),
            "fit-content" => Some(Height::FitContent(None)), // TODO: handle fit-content with length
            "stretch" => Some(Height::Stretch),
            _ => None,
        };

        if let Some(basic_value) = basic {
            return Some(basic_value);
        }

        if let Some(global) = Global::parse(value) {
            return Some(Height::Global(global));
        }

        if value.contains('%')
            && let Some(percentage) = PropertyResolver::resolve_percentage(value)
        {
            return Some(Height::Percentage(percentage));
        }

        if let Some(length) = PropertyResolver::resolve_length(value) {
            return Some(Height::Length(length));
        }

        None
    }

    pub fn resolve_length(value: &str) -> Option<Length> {
        let s = value.trim();
        let split_idx = s.find(|c: char| c.is_alphabetic()).unwrap_or(s.len());
        let (value_str, unit_str) = s.split_at(split_idx);

        let value = value_str.trim().parse::<f32>().ok()?;
        let unit = LengthUnit::from(unit_str);

        Some(Length::new(value, unit))
    }

    fn resolve_percentage(value: &str) -> Option<f32> {
        if let Some(stripped) = value.strip_suffix('%')
            && let Ok(num) = stripped.trim().parse::<f32>()
        {
            return Some(num / 100.0);
        }
        None
    }

    pub fn resolve_line_height(value: &str) -> Option<LineHeight> {
        let global = Global::parse(value);
        if let Some(global_value) = global {
            return Some(LineHeight::Global(global_value));
        }

        if value == "normal" {
            return Some(LineHeight::Normal);
        }

        if let Ok(number) = value.parse::<f32>() {
            return Some(LineHeight::Number(number));
        }

        if let Some(length) = PropertyResolver::resolve_length(value) {
            return Some(LineHeight::Length(length));
        }

        if let Some(percentage) = PropertyResolver::resolve_percentage(value) {
            return Some(LineHeight::Percentage(percentage));
        }

        None
    }

    pub fn resolve_margin(value: &str) -> Option<Margin> {
        let parts = value.split_whitespace().collect::<Vec<&str>>();

        let parse_padding_value = |s: &str| -> Option<MarginValue> {
            if let Some(length) = PropertyResolver::resolve_length(s) {
                return Some(MarginValue::Length(length));
            }

            if let Some(percentage) = PropertyResolver::resolve_percentage(s) {
                return Some(MarginValue::Percentage(percentage));
            }

            if let Some(global) = Global::parse(s) {
                return Some(MarginValue::Global(global));
            }

            if s == "auto" {
                return Some(MarginValue::Auto);
            }

            None
        };

        match parts.len() {
            1 => {
                let value = parse_padding_value(parts[0])?;
                Some(Margin::all(value))
            }
            2 => {
                let vertical = parse_padding_value(parts[0])?;
                let horizontal = parse_padding_value(parts[1])?;
                Some(Margin::two(vertical, horizontal))
            }
            3 => {
                let top = parse_padding_value(parts[0])?;
                let horizontal = parse_padding_value(parts[1])?;
                let bottom = parse_padding_value(parts[2])?;
                Some(Margin::three(top, horizontal, bottom))
            }
            4 => {
                let top = parse_padding_value(parts[0])?;
                let right = parse_padding_value(parts[1])?;
                let bottom = parse_padding_value(parts[2])?;
                let left = parse_padding_value(parts[3])?;
                Some(Margin::new(top, right, bottom, left))
            }
            _ => None,
        }
    }

    pub fn resolve_margin_block(value: &str) -> Option<Margin> {
        let parts = value.split_whitespace().collect::<Vec<&str>>();

        let parse_margin_value = |s: &str| -> Option<MarginValue> {
            if let Some(length) = PropertyResolver::resolve_length(s) {
                return Some(MarginValue::Length(length));
            }

            if let Some(percentage) = PropertyResolver::resolve_percentage(s) {
                return Some(MarginValue::Percentage(percentage));
            }

            if let Some(global) = Global::parse(s) {
                return Some(MarginValue::Global(global));
            }

            if s == "auto" {
                return Some(MarginValue::Auto);
            }

            None
        };

        match parts.len() {
            1 => {
                let value = parse_margin_value(parts[0])?;
                Some(Margin::block(value))
            }
            2 => {
                let start = parse_margin_value(parts[0])?;
                let end = parse_margin_value(parts[1])?;
                Some(Margin::block_two(start, end))
            }
            _ => None,
        }
    }

    pub fn resolve_padding(value: &str) -> Option<Padding> {
        let parts = value.split_whitespace().collect::<Vec<&str>>();

        let parse_padding_value = |s: &str| -> Option<PaddingValue> {
            if let Some(length) = PropertyResolver::resolve_length(s) {
                return Some(PaddingValue::Length(length));
            }

            if let Some(percentage) = PropertyResolver::resolve_percentage(s) {
                return Some(PaddingValue::Percentage(percentage));
            }

            if let Some(global) = Global::parse(s) {
                return Some(PaddingValue::Global(global));
            }

            if s == "auto" {
                return Some(PaddingValue::Auto);
            }

            None
        };

        match parts.len() {
            1 => {
                let value = parse_padding_value(parts[0])?;
                Some(Padding::all(value))
            }
            2 => {
                let vertical = parse_padding_value(parts[0])?;
                let horizontal = parse_padding_value(parts[1])?;
                Some(Padding::two(vertical, horizontal))
            }
            3 => {
                let top = parse_padding_value(parts[0])?;
                let horizontal = parse_padding_value(parts[1])?;
                let bottom = parse_padding_value(parts[2])?;
                Some(Padding::three(top, horizontal, bottom))
            }
            4 => {
                let top = parse_padding_value(parts[0])?;
                let right = parse_padding_value(parts[1])?;
                let bottom = parse_padding_value(parts[2])?;
                let left = parse_padding_value(parts[3])?;
                Some(Padding::new(top, right, bottom, left))
            }
            _ => None,
        }
    }

    pub fn resolve_position(value: &str) -> Option<Position> {
        let global = Global::parse(value);
        if let Some(global_value) = global {
            return Some(Position::Global(global_value));
        }

        match value {
            "static" => Some(Position::Static),
            "relative" => Some(Position::Relative),
            "absolute" => Some(Position::Absolute),
            "fixed" => Some(Position::Fixed),
            "sticky" => Some(Position::Sticky),
            _ => None,
        }
    }

    pub fn resolve_text_align(value: &str) -> Option<TextAlign> {
        let global = Global::parse(value);
        if let Some(global_value) = global {
            return Some(TextAlign::Global(global_value));
        }

        match value {
            "start" => Some(TextAlign::Start),
            "end" => Some(TextAlign::End),
            "left" => Some(TextAlign::Left),
            "right" => Some(TextAlign::Right),
            "center" => Some(TextAlign::Center),
            "justify" => Some(TextAlign::Justify),
            "match-parent" => Some(TextAlign::MatchParent),
            _ => None,
        }
    }

    pub fn resolve_width(value: &str) -> Option<Width> {
        let basic = match value {
            "auto" => Some(Width::Auto),
            "max-content" => Some(Width::MaxContent),
            "min-content" => Some(Width::MinContent),
            "fit-content" => Some(Width::FitContent(None)), // TODO: handle fit-content with length
            "stretch" => Some(Width::Stretch),
            _ => None,
        };

        if let Some(basic_value) = basic {
            return Some(basic_value);
        }

        if let Some(global) = Global::parse(value) {
            return Some(Width::Global(global));
        }

        if value.contains('%')
            && let Some(percentage) = PropertyResolver::resolve_percentage(value)
        {
            return Some(Width::Percentage(percentage));
        }

        if let Some(length) = PropertyResolver::resolve_length(value) {
            return Some(Width::Length(length));
        }

        None
    }
}
