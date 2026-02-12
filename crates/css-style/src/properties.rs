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
pub enum RelativeType {
    FontSize,
    ParentWidth,
    ParentHeight,
    RootFontSize,
    ViewportWidth,
    ViewportHeight,
}

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub struct AbsoluteContext {
    pub root_font_size: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelativeContext {
    pub parent_width: f32,
    pub parent_height: f32,
    pub parent_font_size: f32,
}

impl Default for RelativeContext {
    fn default() -> Self {
        RelativeContext {
            parent_width: 0.0,
            parent_height: 0.0,
            parent_font_size: 16.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CSSProperty<T> {
    Value(T),
    Global(Global),
}

impl<T: FromStr<Err = String>> CSSProperty<T> {
    pub fn as_value_owned(self) -> Option<T> {
        match self {
            CSSProperty::Value(val) => Some(val),
            CSSProperty::Global(_) => None,
        }
    }

    pub fn as_value_ref(&self) -> Option<&T> {
        match self {
            CSSProperty::Value(val) => Some(val),
            CSSProperty::Global(_) => None,
        }
    }

    pub fn resolve(property: &CSSProperty<T>) -> Result<&T, String> {
        match property {
            CSSProperty::Value(val) => Ok(val),
            CSSProperty::Global(global) => {
                Err(format!("Cannot resolve global property: {:?}", global))
            }
        }
    }

    pub fn resolve_with_context<'a>(&'a self, parent: Option<&'a T>, inital: &'a T) -> &'a T {
        match self {
            CSSProperty::Global(global) => match global {
                Global::Initial => inital,
                Global::Inherit => parent.unwrap_or(inital),
                Global::Unset => parent.unwrap_or(inital),
                Global::Revert | Global::RevertLayer => inital, // TODO: Implement user styles
            },
            CSSProperty::Value(val) => val,
        }
    }

    pub fn update_property(property: &mut CSSProperty<T>, value: &str) -> Result<(), String> {
        let new_value = value.parse::<CSSProperty<T>>()?;
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

impl<T> From<T> for CSSProperty<T> {
    fn from(value: T) -> Self {
        CSSProperty::Value(value)
    }
}

impl<T> FromStr for CSSProperty<T>
where
    T: FromStr<Err = String>,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(global) = s.parse::<Global>() {
            Ok(CSSProperty::Global(global))
        } else {
            T::from_str(s)
                .map(CSSProperty::Value)
                .map_err(|e| format!("{:?}", e))
        }
    }
}

// Border
pub type BorderWidthValueProperty = CSSProperty<BorderWidthValue>;
pub type BorderStyleValueProperty = CSSProperty<BorderStyleValue>;

// Color
pub type ColorProperty = CSSProperty<Color>;

// Dimensions
pub type HeightProperty = CSSProperty<Dimension>;
pub type MaxHeightProperty = CSSProperty<MaxDimension>;
pub type WidthProperty = CSSProperty<Dimension>;
pub type MaxWidthProperty = CSSProperty<MaxDimension>;

// Display
pub type DisplayProperty = CSSProperty<Display>;

// Font
pub type FontWeightProperty = CSSProperty<FontWeight>;
pub type FontFamilyProperty = CSSProperty<FontFamily>;
pub type FontSizeProperty = CSSProperty<FontSize>;

// Margin & Padding
pub type OffsetValueProperty = CSSProperty<OffsetValue>;

// Position
pub type PositionProperty = CSSProperty<Position>;

// Text
pub type LineHeightProperty = CSSProperty<LineHeight>;
pub type TextAlignProperty = CSSProperty<TextAlign>;
pub type WritingModeProperty = CSSProperty<WritingMode>;
pub type WhitespaceProperty = CSSProperty<Whitespace>;
