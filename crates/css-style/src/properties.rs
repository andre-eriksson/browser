use css_cssom::ComponentValue;

use crate::{
    BorderStyle, BorderWidth, ComputedStyle, OffsetValue,
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
    pub root_color: Color,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RelativeContext {
    pub parent_style: Box<ComputedStyle>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CSSProperty<T> {
    Value(T),
    Global(Global),
}

impl<T: for<'a> TryFrom<&'a [ComponentValue], Error = String>> CSSProperty<T> {
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

    pub fn update_property(
        property: &mut CSSProperty<T>,
        value: &[ComponentValue],
    ) -> Result<(), String> {
        *property = CSSProperty::from(T::try_from(value)?);
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

// Border
pub type BorderWidthValueProperty = CSSProperty<BorderWidth>;
pub type BorderStyleValueProperty = CSSProperty<BorderStyle>;

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
