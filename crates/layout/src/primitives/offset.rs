use css_style::ComputedMargin;

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

    pub fn vertical(&self) -> f64 {
        self.top.to_px() + self.bottom.to_px()
    }

    pub fn horizontal(&self) -> f64 {
        self.left.to_px() + self.right.to_px()
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
