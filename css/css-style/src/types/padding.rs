use crate::types::{
    global::Global,
    length::{Length, LengthUnit},
};

#[derive(Clone, Debug)]
pub enum PaddingValue {
    Percentage(f32),
    Length(Length),
    Global(Global),
    Auto,
}

#[derive(Clone, Debug)]
pub struct Padding {
    pub top: PaddingValue,
    pub right: PaddingValue,
    pub bottom: PaddingValue,
    pub left: PaddingValue,
}

impl Padding {
    pub fn new(
        top: PaddingValue,
        right: PaddingValue,
        bottom: PaddingValue,
        left: PaddingValue,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn zero() -> Self {
        Self::all(PaddingValue::Length(Length {
            value: 0.0,
            unit: LengthUnit::Px,
        }))
    }

    /// Set all paddings to the same value
    pub fn all(value: PaddingValue) -> Self {
        Self {
            top: value.clone(),
            right: value.clone(),
            bottom: value.clone(),
            left: value,
        }
    }

    /// Set vertical and horizontal paddings
    pub fn two(vertical: PaddingValue, horizontal: PaddingValue) -> Self {
        Self {
            top: vertical.clone(),
            right: horizontal.clone(),
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Set top, horizontal, and bottom paddings
    pub fn three(top: PaddingValue, horizontal: PaddingValue, bottom: PaddingValue) -> Self {
        Self {
            top,
            right: horizontal.clone(),
            bottom,
            left: horizontal,
        }
    }
}
