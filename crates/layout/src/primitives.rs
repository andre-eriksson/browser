use std::ops::Add;

use css_style::ComputedMargin;

/// Rectangle representation for layout dimensions and positions
#[derive(Debug, Default, Clone, Copy)]
pub struct Rect<T = f64> {
    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T,
}

impl<T: Add<Output = T> + PartialOrd + Copy> Rect<T> {
    #[must_use]
    pub const fn new(x: T, y: T, width: T, height: T) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub fn contains_point(&self, px: T, py: T) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarginValue {
    Auto,
    Px(f64),
}

impl MarginValue {
    pub fn resolve(computed: ComputedMargin, containing_width: f64) -> Self {
        match computed {
            ComputedMargin::Auto => Self::Auto,
            ComputedMargin::Px(px) => Self::Px(px),
            ComputedMargin::Percentage(frac) => {
                let px = frac * containing_width;
                Self::Px(px)
            }
        }
    }

    pub fn to_px(self) -> f64 {
        match self {
            Self::Auto => 0.0,
            Self::Px(px) => px,
        }
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }
}

impl Default for MarginValue {
    fn default() -> Self {
        Self::Px(0f64)
    }
}

impl From<f64> for MarginValue {
    fn from(value: f64) -> Self {
        Self::Px(value)
    }
}

/// Resolved edge values (border, margins, padding) in pixels
#[derive(Debug, Clone, Copy, Default)]
pub struct Margin {
    pub top: MarginValue,
    pub right: MarginValue,
    pub bottom: MarginValue,
    pub left: MarginValue,
}

impl Margin {
    #[must_use]
    pub fn all(value: f64) -> Self {
        Self {
            top: value.into(),
            right: value.into(),
            bottom: value.into(),
            left: value.into(),
        }
    }

    #[must_use]
    pub fn zero() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SideOffset {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl SideOffset {
    #[must_use]
    pub fn all(value: f64) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn vertical(&self) -> f64 {
        self.top + self.bottom
    }

    pub fn horizontal(&self) -> f64 {
        self.left + self.right
    }

    #[must_use]
    pub fn zero() -> Self {
        Self::default()
    }
}
