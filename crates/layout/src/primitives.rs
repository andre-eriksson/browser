use std::ops::Add;

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

/// Resolved edge values (border, margins, padding) in pixels
#[derive(Debug, Clone, Copy, Default)]
pub struct SideOffset {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl SideOffset {
    #[must_use]
    pub const fn all(value: f64) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    #[must_use]
    pub fn horizontal(&self) -> f64 {
        self.left + self.right
    }

    #[must_use]
    pub fn vertical(&self) -> f64 {
        self.top + self.bottom
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }
}
