use std::str::FromStr;

use crate::{
    primitives::global::Global,
    properties::{
        border::{BorderColor, BorderStyle, BorderWidth},
        color::Color,
        dimension::{Dimension, MaxDimension},
        display::Display,
        font::{FontFamily, FontSize, FontWeight},
        offset::Offset,
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

    pub fn update_property_field<U>(
        property: &mut Property<T>,
        field_selector: impl FnOnce(&mut T) -> &mut U,
        value: &str,
    ) -> Result<(), String>
    where
        U: FromStr<Err = String>,
    {
        match property {
            Property::Value(val) => {
                let field = field_selector(val);
                Property::update_property(field, value)
            }
            Property::Global(global) => {
                let new_global = value.parse::<Global>()?;
                *global = new_global;
                Ok(())
            }
        }
    }

    pub fn update_property_fields<U>(
        property: &mut Property<T>,
        field_selectors: Vec<impl FnOnce(&mut T) -> &mut U>,
        values: Vec<&str>,
    ) -> Result<(), String>
    where
        T: FromStr<Err = String>,
        U: FromStr<Err = String>,
    {
        if field_selectors.len() != values.len() {
            return Err("Field selectors and values length mismatch".to_string());
        }

        match property {
            Property::Value(val) => {
                for (selector, value) in field_selectors.into_iter().zip(values.into_iter()) {
                    let field = selector(val);
                    Property::update_property(field, value)?;
                }
                Ok(())
            }
            Property::Global(global) => {
                if values.len() != 1 {
                    return Err("Multiple values provided for global property".to_string());
                }
                let new_global = values[0].parse::<Global>()?;
                *global = new_global;
                Ok(())
            }
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
pub type BorderWidthProperty = Property<BorderWidth>;
pub type BorderColorProperty = Property<BorderColor>;
pub type BorderStyleProperty = Property<BorderStyle>;

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
pub type OffsetProperty = Property<Offset>;

// Position
pub type PositionProperty = Property<Position>;

// Text
pub type LineHeightProperty = Property<LineHeight>;
pub type TextAlignProperty = Property<TextAlign>;
pub type WritingModeProperty = Property<WritingMode>;
pub type WhitespaceProperty = Property<Whitespace>;
