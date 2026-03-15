//! This module defines the `OffsetValue` and `Offset` types, which represent CSS offset values for properties like margin and padding.
//! An `OffsetValue` can be a length, percentage, calc expression, or auto. An `Offset` represents the four sides (top, right, bottom, left)
//! of a CSS property, each with its own `OffsetValue`. The module also includes methods to convert these values to pixels based on the
//! relative and absolute contexts.

use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};
use css_values::{
    calc::{CalcExpression, is_math_function},
    dimension::OffsetValue,
    error::CssValueError,
    numeric::Percentage,
    quantity::{Length, LengthUnit},
};

use crate::properties::{AbsoluteContext, CSSParsable, PixelRepr, RelativeContext, RelativeType};

impl PixelRepr for OffsetValue {
    fn to_px(
        &self,
        rel_type: Option<RelativeType>,
        rel_ctx: Option<&RelativeContext>,
        abs_ctx: &AbsoluteContext,
    ) -> f32 {
        match self {
            OffsetValue::Length(len) => len.to_px(rel_type, rel_ctx, abs_ctx),
            OffsetValue::Percentage(pct) => match rel_type {
                Some(RelativeType::FontSize) => rel_ctx
                    .map(|ctx| ctx.font_size * pct.as_fraction())
                    .unwrap_or(abs_ctx.root_font_size * pct.as_fraction()),
                Some(RelativeType::ParentHeight) => rel_ctx
                    .map(|ctx| ctx.parent.intrinsic_height * pct.as_fraction())
                    .unwrap_or(abs_ctx.viewport_height * pct.as_fraction()),
                Some(RelativeType::ParentWidth) => rel_ctx
                    .map(|ctx| ctx.parent.intrinsic_width * pct.as_fraction())
                    .unwrap_or(abs_ctx.viewport_width * pct.as_fraction()),
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
    type Error = CssValueError;

    fn try_from(values: &[OffsetValue]) -> Result<Self, Self::Error> {
        match values.len() {
            1 => Ok(Offset::all(values[0].clone())),
            2 => Ok(Offset::vh(values[0].clone(), values[1].clone())),
            3 => Ok(Offset::thb(values[0].clone(), values[1].clone(), values[2].clone())),
            4 => Ok(Offset::trbl(values[0].clone(), values[1].clone(), values[2].clone(), values[3].clone())),
            _ => Err(CssValueError::InvalidValue(format!("Expected 1 to 4 offset values, but got {}", values.len()))),
        }
    }
}

impl CSSParsable for Offset {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();
        let mut offset_values = Vec::new();

        while let Some(cv) = stream.next_cv() {
            match cv {
                ComponentValue::Function(func) if is_math_function(&func.name) => {
                    offset_values
                        .push(OffsetValue::Calc(CalcExpression::parse_math_function(&func.name, &func.value)?));
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;
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
                _ => return Err(CssValueError::InvalidComponentValue(cv.clone())),
            }
        }

        if offset_values.is_empty() || offset_values.len() > 4 {
            return Err(CssValueError::InvalidValue(format!(
                "Expected 1 to 4 offset values, but got {}",
                offset_values.len()
            )));
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

        let offset = Offset::parse(&mut input.as_slice().into()).unwrap();
        assert_eq!(offset.top, OffsetValue::Length(Length::px(10.0)));
        assert_eq!(offset.right, OffsetValue::Percentage(Percentage::new(50.0)));
        assert_eq!(offset.bottom, OffsetValue::Length(Length::px(10.0)));
        assert_eq!(offset.left, OffsetValue::Percentage(Percentage::new(50.0)));
    }
}
