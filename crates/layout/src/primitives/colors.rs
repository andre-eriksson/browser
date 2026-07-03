use css_style::{Color4f, ComputedStyle};

#[derive(Debug, Clone)]
pub struct BorderColor {
    pub top: Color4f,
    pub right: Color4f,
    pub bottom: Color4f,
    pub left: Color4f,
}

impl Default for BorderColor {
    fn default() -> Self {
        Self {
            top: Color4f::BLACK,
            right: Color4f::BLACK,
            bottom: Color4f::BLACK,
            left: Color4f::BLACK,
        }
    }
}

/// Color properties extracted for rendering
#[derive(Debug, Clone)]
pub struct LayoutColors {
    /// The background color of the layout node
    pub background_color: Color4f,

    /// Text color of the layout node
    pub color: Color4f,

    /// Border color of the layout node
    pub border_color: BorderColor,
}

impl LayoutColors {
    /// Creates colors for a text node using only the inherited foreground color.
    /// Background and border are transparent since those come from `InlineDecoration`.
    #[must_use]
    pub fn text_only(color: Color4f) -> Self {
        Self {
            background_color: Color4f::TRANSPARENT,
            color,
            border_color: BorderColor::default(),
        }
    }
}

impl Default for LayoutColors {
    fn default() -> Self {
        Self {
            background_color: Color4f::TRANSPARENT,
            color: Color4f::BLACK,
            border_color: BorderColor::default(),
        }
    }
}

impl From<&ComputedStyle> for LayoutColors {
    fn from(style: &ComputedStyle) -> Self {
        Self {
            background_color: style.background_color,
            color: style.color,
            border_color: BorderColor {
                top: style.border_top_color,
                right: style.border_right_color,
                bottom: style.border_bottom_color,
                left: style.border_left_color,
            },
        }
    }
}
