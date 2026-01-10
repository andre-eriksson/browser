use crate::types::{
    color::{Color, NamedColor},
    global::Global,
    length::Length,
};

#[derive(Debug, Clone)]
pub enum BorderWidthValue {
    Length(Length),
    Thin,
    Medium,
    Thick,
    Global(Global),
}

#[derive(Clone, Debug)]
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
            top: value.clone(),
            right: value.clone(),
            bottom: value.clone(),
            left: value,
        }
    }

    /// Set vertical and horizontal border widths
    pub fn two(vertical: BorderWidthValue, horizontal: BorderWidthValue) -> Self {
        Self {
            top: vertical.clone(),
            right: horizontal.clone(),
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
            right: horizontal.clone(),
            bottom,
            left: horizontal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

impl BorderStyleValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "none" => Some(BorderStyleValue::None),
            "hidden" => Some(BorderStyleValue::Hidden),
            "dotted" => Some(BorderStyleValue::Dotted),
            "dashed" => Some(BorderStyleValue::Dashed),
            "solid" => Some(BorderStyleValue::Solid),
            "double" => Some(BorderStyleValue::Double),
            "groove" => Some(BorderStyleValue::Groove),
            "ridge" => Some(BorderStyleValue::Ridge),
            "inset" => Some(BorderStyleValue::Inset),
            "outset" => Some(BorderStyleValue::Outset),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
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
            top: value.clone(),
            right: value.clone(),
            bottom: value.clone(),
            left: value,
        }
    }

    /// Set vertical and horizontal border styles
    pub fn two(vertical: BorderStyleValue, horizontal: BorderStyleValue) -> Self {
        Self {
            top: vertical.clone(),
            right: horizontal.clone(),
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
            right: horizontal.clone(),
            bottom,
            left: horizontal,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BorderColorValue {
    Color(Color),
    Global(Global),
}

#[derive(Clone, Debug)]
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
            top: value.clone(),
            right: value.clone(),
            bottom: value.clone(),
            left: value,
        }
    }

    /// Set vertical and horizontal border colors
    pub fn two(vertical: BorderColorValue, horizontal: BorderColorValue) -> Self {
        Self {
            top: vertical.clone(),
            right: horizontal.clone(),
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
            right: horizontal.clone(),
            bottom,
            left: horizontal,
        }
    }
}

#[derive(Clone, Debug)]
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
