use crate::types::{global::Global, length::Length};

#[derive(Clone, Debug)]
pub enum MarginValue {
    Percentage(f32),
    Length(Length),
    Global(Global),
    Auto,
}

#[derive(Clone, Debug)]
pub struct Margin {
    pub top: MarginValue,
    pub right: MarginValue,
    pub bottom: MarginValue,
    pub left: MarginValue,
}

impl Margin {
    pub fn new(
        top: MarginValue,
        right: MarginValue,
        bottom: MarginValue,
        left: MarginValue,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Set all margins to the same value
    pub fn all(value: MarginValue) -> Self {
        Self {
            top: value.clone(),
            right: value.clone(),
            bottom: value.clone(),
            left: value,
        }
    }

    /// Set vertical and horizontal margins
    pub fn two(vertical: MarginValue, horizontal: MarginValue) -> Self {
        Self {
            top: vertical.clone(),
            right: horizontal.clone(),
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Set top, horizontal, and bottom margins
    pub fn three(top: MarginValue, horizontal: MarginValue, bottom: MarginValue) -> Self {
        Self {
            top,
            right: horizontal.clone(),
            bottom,
            left: horizontal,
        }
    }
}
