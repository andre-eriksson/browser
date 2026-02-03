use std::str::FromStr;

use crate::{
    BorderStyleValue, BorderWidthValue, OffsetValue,
    primitives::global::Global,
    properties::{
        color::Color,
        dimension::{Dimension, MaxDimension},
        display::Display,
        font::{FontFamily, FontSize, FontWeight},
        position::Position,
        text::{LineHeight, TextAlign, Whitespace, WritingMode},
    },
};

pub mod border;
pub mod color;
pub mod dimension;
pub mod display;
pub mod font;
pub mod offset;
pub mod position;
pub mod text;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Property<T> {
    Value(T),
    Global(Global),
}

impl<T: FromStr<Err = String>> Property<T> {
    pub fn wrap_value(value: T) -> Self {
        Property::Value(value)
    }

    pub fn resolve(property: &Property<T>) -> Result<&T, String> {
        match property {
            Property::Value(val) => Ok(val),
            Property::Global(global) => {
                Err(format!("Cannot resolve global property: {:?}", global))
            }
        }
    }

    pub fn update_property(property: &mut T, value: &str) -> Result<(), String> {
        let new_value = value.parse::<T>()?;
        *property = new_value;
        Ok(())
    }

    pub fn update(property: &mut T, value: T) {
        *property = value;
    }

    pub fn update_multiple(properties: &mut [&mut T], value: T)
    where
        T: Clone,
    {
        for property in properties {
            **property = value.clone();
        }
    }
}

impl<T> From<T> for Property<T> {
    fn from(value: T) -> Self {
        Property::Value(value)
    }
}

impl<T> FromStr for Property<T>
where
    T: FromStr<Err = String>,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(global) = s.parse::<Global>() {
            Ok(Property::Global(global))
        } else {
            T::from_str(s)
                .map(Property::Value)
                .map_err(|e| format!("{:?}", e))
        }
    }
}

// Border
pub type BorderWidthValueProperty = Property<BorderWidthValue>;
pub type BorderStyleValueProperty = Property<BorderStyleValue>;

// Color
pub type ColorProperty = Property<Color>;

// Dimensions
pub type HeightProperty = Property<Dimension>;
pub type MaxHeightProperty = Property<MaxDimension>;
pub type WidthProperty = Property<Dimension>;
pub type MaxWidthProperty = Property<MaxDimension>;

// Display
pub type DisplayProperty = Property<Display>;

// Font
pub type FontWeightProperty = Property<FontWeight>;
pub type FontFamilyProperty = Property<FontFamily>;
pub type FontSizeProperty = Property<FontSize>;

// Margin & Padding
pub type OffsetValueProperty = Property<OffsetValue>;

// Position
pub type PositionProperty = Property<Position>;

// Text
pub type LineHeightProperty = Property<LineHeight>;
pub type TextAlignProperty = Property<TextAlign>;
pub type WritingModeProperty = Property<WritingMode>;
pub type WhitespaceProperty = Property<Whitespace>;
