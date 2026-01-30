use crate::types::{
    Parseable,
    color::{Color, NamedColor},
    global::Global,
    length::Length,
};

#[derive(Debug, Clone, Copy)]
pub enum BorderWidthValue {
    Length(Length),
    Thin,
    Medium,
    Thick,
    Global(Global),
}

#[derive(Debug, Clone)]
pub struct BorderWidth {
    pub top: BorderWidthValue,
    pub right: BorderWidthValue,
    pub bottom: BorderWidthValue,
    pub left: BorderWidthValue,
}

impl BorderWidth {
    pub fn new(
        top: BorderWidthValue,
        right: BorderWidthValue,
        bottom: BorderWidthValue,
        left: BorderWidthValue,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Set all border widths to the same value
    pub fn all(value: BorderWidthValue) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Set vertical and horizontal border widths
    pub fn two(vertical: BorderWidthValue, horizontal: BorderWidthValue) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Set top, horizontal, and bottom border widths
    pub fn three(
        top: BorderWidthValue,
        horizontal: BorderWidthValue,
        bottom: BorderWidthValue,
    ) -> Self {
        Self {
            top,
            right: horizontal,
            bottom,
            left: horizontal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BorderStyleValue {
    None,
    Hidden,
    Dotted,
    Dashed,
    Solid,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
    Global(Global),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorderStyle {
    pub top: BorderStyleValue,
    pub right: BorderStyleValue,
    pub bottom: BorderStyleValue,
    pub left: BorderStyleValue,
}

impl BorderStyle {
    pub fn new(
        top: BorderStyleValue,
        right: BorderStyleValue,
        bottom: BorderStyleValue,
        left: BorderStyleValue,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Set all border styles to the same value
    pub fn all(value: BorderStyleValue) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Set vertical and horizontal border styles
    pub fn two(vertical: BorderStyleValue, horizontal: BorderStyleValue) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Set top, horizontal, and bottom border styles
    pub fn three(
        top: BorderStyleValue,
        horizontal: BorderStyleValue,
        bottom: BorderStyleValue,
    ) -> Self {
        Self {
            top,
            right: horizontal,
            bottom,
            left: horizontal,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BorderColorValue {
    Color(Color),
    Global(Global),
}

#[derive(Debug, Clone)]
pub struct BorderColor {
    pub top: BorderColorValue,
    pub right: BorderColorValue,
    pub bottom: BorderColorValue,
    pub left: BorderColorValue,
}

impl BorderColor {
    pub fn new(
        top: BorderColorValue,
        right: BorderColorValue,
        bottom: BorderColorValue,
        left: BorderColorValue,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Set all border colors to the same value
    pub fn all(value: BorderColorValue) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Set vertical and horizontal border colors
    pub fn two(vertical: BorderColorValue, horizontal: BorderColorValue) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Set top, horizontal, and bottom border colors
    pub fn three(
        top: BorderColorValue,
        horizontal: BorderColorValue,
        bottom: BorderColorValue,
    ) -> Self {
        Self {
            top,
            right: horizontal,
            bottom,
            left: horizontal,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Border {
    pub width: BorderWidth,
    pub style: BorderStyle,
    pub color: BorderColor,
}

impl Border {
    pub fn none() -> Self {
        Border {
            width: BorderWidth::all(BorderWidthValue::Medium),
            style: BorderStyle::all(BorderStyleValue::None),
            color: BorderColor::all(BorderColorValue::Color(Color::Named(NamedColor::Black))),
        }
    }
}

impl Parseable for BorderStyleValue {
    fn parse(value: &str) -> Option<Self> {
        match value.len() {
            4 if value.eq_ignore_ascii_case("none") => Some(Self::None),
            5 => {
                if value.eq_ignore_ascii_case("ridge") {
                    Some(Self::Ridge)
                } else if value.eq_ignore_ascii_case("inset") {
                    Some(Self::Inset)
                } else if value.eq_ignore_ascii_case("solid") {
                    Some(Self::Solid)
                } else {
                    None
                }
            }
            6 => {
                if value.eq_ignore_ascii_case("hidden") {
                    Some(Self::Hidden)
                } else if value.eq_ignore_ascii_case("dotted") {
                    Some(Self::Dotted)
                } else if value.eq_ignore_ascii_case("dashed") {
                    Some(Self::Dashed)
                } else if value.eq_ignore_ascii_case("double") {
                    Some(Self::Double)
                } else if value.eq_ignore_ascii_case("groove") {
                    Some(Self::Groove)
                } else if value.eq_ignore_ascii_case("outset") {
                    Some(Self::Outset)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Parseable for Border {
    fn parse(value: &str) -> Option<Self> {
        let mut border = Self::none();

        let parts = value.split_whitespace().collect::<Vec<&str>>();

        // Only supporting: '<length> <style> <color>' for now

        for part in parts {
            if let Some(length) = Length::parse(part) {
                border.width = BorderWidth::all(BorderWidthValue::Length(length))
            }

            if let Some(style) = BorderStyleValue::parse(value) {
                border.style = BorderStyle::all(style);
            }

            if let Some(color) = Color::parse(part) {
                border.color = BorderColor::all(BorderColorValue::Color(color));
            }
        }

        Some(border)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_border_width_all() {
        let border_width = BorderWidth::all(BorderWidthValue::Thin);
        assert!(matches!(border_width.top, BorderWidthValue::Thin));
        assert!(matches!(border_width.right, BorderWidthValue::Thin));
        assert!(matches!(border_width.bottom, BorderWidthValue::Thin));
        assert!(matches!(border_width.left, BorderWidthValue::Thin));
    }

    #[test]
    fn test_border_style_parse() {
        assert_eq!(
            BorderStyleValue::parse("none"),
            Some(BorderStyleValue::None)
        );
        assert_eq!(
            BorderStyleValue::parse("hidden"),
            Some(BorderStyleValue::Hidden)
        );
        assert_eq!(
            BorderStyleValue::parse("dotted"),
            Some(BorderStyleValue::Dotted)
        );
        assert_eq!(
            BorderStyleValue::parse("dashed"),
            Some(BorderStyleValue::Dashed)
        );
        assert_eq!(
            BorderStyleValue::parse("solid"),
            Some(BorderStyleValue::Solid)
        );
        assert_eq!(
            BorderStyleValue::parse("double"),
            Some(BorderStyleValue::Double)
        );
        assert_eq!(
            BorderStyleValue::parse("groove"),
            Some(BorderStyleValue::Groove)
        );
        assert_eq!(
            BorderStyleValue::parse("ridge"),
            Some(BorderStyleValue::Ridge)
        );
        assert_eq!(
            BorderStyleValue::parse("inset"),
            Some(BorderStyleValue::Inset)
        );
        assert_eq!(
            BorderStyleValue::parse("outset"),
            Some(BorderStyleValue::Outset)
        );
    }

    #[test]
    fn test_border_style_case_parse() {
        assert_eq!(
            BorderStyleValue::parse("noNE"),
            Some(BorderStyleValue::None)
        );
        assert_eq!(
            BorderStyleValue::parse("HidDen"),
            Some(BorderStyleValue::Hidden)
        );
    }
}
