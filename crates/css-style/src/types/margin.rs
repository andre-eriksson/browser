use crate::types::{
    global::Global,
    length::{Length, LengthUnit},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarginValue {
    Percentage(f32),
    Length(Length),
    Global(Global),
    Auto,
}

#[derive(Debug, Clone, PartialEq)]
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

    pub fn zero() -> Self {
        Self::all(MarginValue::Length(Length {
            value: 0.0,
            unit: LengthUnit::Px,
        }))
    }

    pub fn block(value: MarginValue) -> Self {
        Self {
            top: value,
            right: MarginValue::Length(Length {
                value: 0.0,
                unit: LengthUnit::Px,
            }),
            bottom: value,
            left: MarginValue::Length(Length {
                value: 0.0,
                unit: LengthUnit::Px,
            }),
        }
    }

    pub fn block_two(top: MarginValue, bottom: MarginValue) -> Self {
        Self {
            top,
            right: MarginValue::Length(Length {
                value: 0.0,
                unit: LengthUnit::Px,
            }),
            bottom,
            left: MarginValue::Length(Length {
                value: 0.0,
                unit: LengthUnit::Px,
            }),
        }
    }

    /// Set all margins to the same value
    pub fn all(value: MarginValue) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Set vertical and horizontal margins
    pub fn two(vertical: MarginValue, horizontal: MarginValue) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Set top, horizontal, and bottom margins
    pub fn three(top: MarginValue, horizontal: MarginValue, bottom: MarginValue) -> Self {
        Self {
            top,
            right: horizontal,
            bottom,
            left: horizontal,
        }
    }
}
