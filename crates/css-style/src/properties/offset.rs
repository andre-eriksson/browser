//! This module defines the `OffsetValue` and `Offset` types, which represent CSS offset values for properties like margin and padding.
//! An `OffsetValue` can be a length, percentage, calc expression, or auto. An `Offset` represents the four sides (top, right, bottom, left)
//! of a CSS property, each with its own `OffsetValue`. The module also includes methods to convert these values to pixels based on the
//! relative and absolute contexts.

use css_cssom::{ComponentValue, CssTokenKind};

use crate::{
    functions::calculate::{CalcExpression, is_math_function},
    length::LengthUnit,
    primitives::{length::Length, percentage::Percentage},
    properties::{AbsoluteContext, RelativeContext, RelativeType},
};

/// Represents a CSS offset value, used for specific margin and padding values. It can be a length, percentage, calc expression, or auto.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/padding-top>
#[derive(Debug, Clone, PartialEq)]
pub enum OffsetValue {
    Percentage(Percentage),
    Length(Length),
    Calc(CalcExpression),
    Auto,
}

impl Default for OffsetValue {
    fn default() -> Self {
        OffsetValue::zero()
    }
}

impl OffsetValue {
    pub(crate) fn zero() -> Self {
        Self::Length(Length::zero())
    }

    pub fn px(value: f32) -> Self {
        Self::Length(Length::px(value))
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, OffsetValue::Auto)
    }

    /// Convert the OffsetValue to pixels, given the relative and absolute contexts. The rel_type indicates what the percentage is relative to.
    pub fn to_px(&self, rel_type: Option<RelativeType>, rel_ctx: &RelativeContext, abs_ctx: &AbsoluteContext) -> f32 {
        match self {
            OffsetValue::Length(len) => len.to_px(rel_ctx, abs_ctx),
            OffsetValue::Percentage(pct) => match rel_type {
                Some(RelativeType::FontSize) => rel_ctx.parent.font_size * pct.as_fraction(),
                Some(RelativeType::ParentHeight) => rel_ctx.parent.intrinsic_height * pct.as_fraction(),
                Some(RelativeType::ParentWidth) => rel_ctx.parent.intrinsic_width * pct.as_fraction(),
                Some(RelativeType::RootFontSize) => abs_ctx.root_font_size * pct.as_fraction(),
                Some(RelativeType::ViewportHeight) => abs_ctx.viewport_height * pct.as_fraction(),
                Some(RelativeType::ViewportWidth) => abs_ctx.viewport_width * pct.as_fraction(),
                None => 0.0,
            },
            OffsetValue::Calc(calc) => calc.to_px(rel_type, rel_ctx, abs_ctx),
            OffsetValue::Auto => 0.0,
        }
    }
}

impl TryFrom<&[ComponentValue]> for OffsetValue {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Function(func) if is_math_function(&func.name) => {
                    return Ok(Self::Calc(CalcExpression::parse_math_function(&func.name, func.value.as_slice())?));
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| format!("Invalid length unit: {}", unit))?;
                        return Ok(Self::Length(Length::new(value.to_f64() as f32, len_unit)));
                    }
                    CssTokenKind::Percentage(pct) => {
                        return Ok(Self::Percentage(Percentage::new(pct.to_f64() as f32)));
                    }
                    CssTokenKind::Number(num) => {
                        return Ok(Self::Length(Length::px(num.to_f64() as f32)));
                    }
                    CssTokenKind::Ident(ident) if ident.eq_ignore_ascii_case("auto") => {
                        return Ok(Self::Auto);
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Err("No valid OffsetValue found in ComponentValue list".into())
    }
}

/// Represents the offset values for the four sides (top, right, bottom, left) of a CSS property like margin or padding. Each side can have its own OffsetValue.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/margin>
#[derive(Debug, Clone, PartialEq)]
pub struct Offset {
    pub top: OffsetValue,
    pub right: OffsetValue,
    pub bottom: OffsetValue,
    pub left: OffsetValue,
}

impl Offset {
    /// Create an Offset with individual values for each side (top, right, bottom, left).
    pub(crate) fn trbl(top: OffsetValue, right: OffsetValue, bottom: OffsetValue, left: OffsetValue) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Create an Offset with the same value for all sides.
    pub(crate) fn all(value: OffsetValue) -> Self {
        Self {
            top: value.clone(),
            right: value.clone(),
            bottom: value.clone(),
            left: value,
        }
    }

    /// Create an Offset with one value for vertical sides (top and bottom) and another for horizontal sides (right and left).
    pub(crate) fn vh(vertical: OffsetValue, horizontal: OffsetValue) -> Self {
        Self {
            top: vertical.clone(),
            right: horizontal.clone(),
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Create an Offset with three values: one for the top, one for the horizontal sides (right and left), and one for the bottom.
    pub(crate) fn thb(top: OffsetValue, horizontal: OffsetValue, bottom: OffsetValue) -> Self {
        Self {
            top,
            right: horizontal.clone(),
            bottom,
            left: horizontal,
        }
    }
}

impl TryFrom<&[OffsetValue]> for Offset {
    type Error = String;

    fn try_from(values: &[OffsetValue]) -> Result<Self, Self::Error> {
        match values.len() {
            1 => Ok(Offset::all(values[0].clone())),
            2 => Ok(Offset::vh(values[0].clone(), values[1].clone())),
            3 => Ok(Offset::thb(values[0].clone(), values[1].clone(), values[2].clone())),
            4 => Ok(Offset::trbl(values[0].clone(), values[1].clone(), values[2].clone(), values[3].clone())),
            _ => Err(format!("Invalid number of Offset values: expected 1-4, got {}", values.len())),
        }
    }
}

impl TryFrom<&[ComponentValue]> for Offset {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let mut offset_values = Vec::new();

        for cv in value {
            match cv {
                ComponentValue::Function(func) if is_math_function(&func.name) => {
                    offset_values.push(OffsetValue::Calc(CalcExpression::parse_math_function(
                        &func.name,
                        func.value.as_slice(),
                    )?));
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| format!("Invalid length unit: {}", unit))?;
                        offset_values.push(OffsetValue::Length(Length::new(value.to_f64() as f32, len_unit)));
                    }
                    CssTokenKind::Percentage(pct) => {
                        offset_values.push(OffsetValue::Percentage(Percentage::new(pct.to_f64() as f32)));
                    }
                    CssTokenKind::Number(num) => {
                        offset_values.push(OffsetValue::Length(Length::px(num.to_f64() as f32)));
                    }
                    CssTokenKind::Ident(ident) if ident.eq_ignore_ascii_case("auto") => {
                        offset_values.push(OffsetValue::Auto);
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Offset::try_from(offset_values.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use css_cssom::{CssToken, NumericValue};

    #[test]
    fn test_parse() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Dimension {
                    value: NumericValue::from(10.0),
                    unit: "px".into(),
                },
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Percentage(NumericValue::from(50.0)),
                position: None,
            }),
        ];

        let offset = Offset::try_from(input.as_slice()).unwrap();
        assert_eq!(offset.top, OffsetValue::Length(Length::px(10.0)));
        assert_eq!(offset.right, OffsetValue::Percentage(Percentage::new(50.0)));
        assert_eq!(offset.bottom, OffsetValue::Length(Length::px(10.0)));
        assert_eq!(offset.left, OffsetValue::Percentage(Percentage::new(50.0)));
    }
}
