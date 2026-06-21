use std::ops::Add;

/// Rectangle representation for layout dimensions and positions
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Default, Clone, Copy)]
pub struct Size<T = f64> {
    pub width: T,
    pub height: T,
}

impl<T: Add<Output = T> + PartialOrd + Copy> Size<T> {
    #[must_use]
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}
