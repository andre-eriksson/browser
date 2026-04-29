//! This module defines the `OffsetValue` and `Offset` types, which represent CSS offset values for properties like margin and padding.
//! An `OffsetValue` can be a length, percentage, calc expression, or auto. An `Offset` represents the four sides (top, right, bottom, left)
//! of a CSS property, each with its own `OffsetValue`. The module also includes methods to convert these values to pixels based on the
//! relative and absolute contexts.

use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};
use css_values::{
    calc::{CalcExpression, is_math_function},
    dimension::{MarginValue, OffsetValue},
    error::CssValueError,
    numeric::Percentage,
    quantity::{Length, LengthUnit},
};

use crate::properties::CSSParsable;

pub struct Offset {
    pub top: OffsetValue,
    pub right: OffsetValue,
    pub bottom: OffsetValue,
    pub left: OffsetValue,
}

impl Offset {
    /// Create an Offset with individual values for each side (top, right, bottom, left).
    pub(crate) const fn trbl(top: OffsetValue, right: OffsetValue, bottom: OffsetValue, left: OffsetValue) -> Self {
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
            1 => Ok(Self::all(values[0].clone())),
            2 => Ok(Self::vh(values[0].clone(), values[1].clone())),
            3 => Ok(Self::thb(values[0].clone(), values[1].clone(), values[2].clone())),
            4 => Ok(Self::trbl(values[0].clone(), values[1].clone(), values[2].clone(), values[3].clone())),
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
                        .push(OffsetValue::Calc(CalcExpression::parse(&func.name, &func.value)?));
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;
                        offset_values.push(OffsetValue::Length(Length::new(value.to_f64(), len_unit)));
                    }
                    CssTokenKind::Percentage(pct) => {
                        offset_values.push(OffsetValue::Percentage(Percentage::new(pct.to_f64())));
                    }
                    _ => {}
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

        Self::try_from(offset_values.as_slice())
    }
}

/// Represents the offset values for the four sides (top, right, bottom, left) of a CSS property like margin or padding. Each side can have its own `OffsetValue`.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/margin>
#[derive(Debug, Clone, PartialEq)]
pub struct Margin {
    pub top: MarginValue,
    pub right: MarginValue,
    pub bottom: MarginValue,
    pub left: MarginValue,
}

impl Margin {
    /// Create an Offset with individual values for each side (top, right, bottom, left).
    pub(crate) const fn trbl(top: MarginValue, right: MarginValue, bottom: MarginValue, left: MarginValue) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Create an Offset with the same value for all sides.
    pub(crate) fn all(value: MarginValue) -> Self {
        Self {
            top: value.clone(),
            right: value.clone(),
            bottom: value.clone(),
            left: value,
        }
    }

    /// Create an Offset with one value for vertical sides (top and bottom) and another for horizontal sides (right and left).
    pub(crate) fn vh(vertical: MarginValue, horizontal: MarginValue) -> Self {
        Self {
            top: vertical.clone(),
            right: horizontal.clone(),
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Create an Offset with three values: one for the top, one for the horizontal sides (right and left), and one for the bottom.
    pub(crate) fn thb(top: MarginValue, horizontal: MarginValue, bottom: MarginValue) -> Self {
        Self {
            top,
            right: horizontal.clone(),
            bottom,
            left: horizontal,
        }
    }
}

impl TryFrom<&[MarginValue]> for Margin {
    type Error = CssValueError;

    fn try_from(values: &[MarginValue]) -> Result<Self, Self::Error> {
        match values.len() {
            1 => Ok(Self::all(values[0].clone())),
            2 => Ok(Self::vh(values[0].clone(), values[1].clone())),
            3 => Ok(Self::thb(values[0].clone(), values[1].clone(), values[2].clone())),
            4 => Ok(Self::trbl(values[0].clone(), values[1].clone(), values[2].clone(), values[3].clone())),
            _ => Err(CssValueError::InvalidValue(format!("Expected 1 to 4 offset values, but got {}", values.len()))),
        }
    }
}

impl CSSParsable for Margin {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();
        let mut offset_values = Vec::new();

        while let Some(cv) = stream.next_cv() {
            match cv {
                ComponentValue::Function(func) if is_math_function(&func.name) => {
                    offset_values
                        .push(MarginValue::Calc(CalcExpression::parse(&func.name, &func.value)?));
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;
                        offset_values.push(MarginValue::Length(Length::new(value.to_f64(), len_unit)));
                    }
                    CssTokenKind::Percentage(pct) => {
                        offset_values.push(MarginValue::Percentage(Percentage::new(pct.to_f64())));
                    }
                    CssTokenKind::Ident(ident) if ident.eq_ignore_ascii_case("auto") => {
                        offset_values.push(MarginValue::Auto);
                    }
                    _ => {}
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

        Self::try_from(offset_values.as_slice())
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

        let offset = Margin::parse(&mut input.as_slice().into()).unwrap();
        assert_eq!(offset.top, MarginValue::Length(Length::px(10.0)));
        assert_eq!(offset.right, MarginValue::Percentage(Percentage::new(50.0)));
        assert_eq!(offset.bottom, MarginValue::Length(Length::px(10.0)));
        assert_eq!(offset.left, MarginValue::Percentage(Percentage::new(50.0)));
    }
}
