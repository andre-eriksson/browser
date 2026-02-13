use std::str::FromStr;

use css_cssom::{ComponentValue, CssTokenKind};

use crate::{
    calculate::CalcExpression,
    length::LengthUnit,
    primitives::{length::Length, percentage::Percentage},
    properties::{AbsoluteContext, RelativeContext, RelativeType},
};

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
    pub fn zero() -> Self {
        Self::Length(Length::zero())
    }

    pub fn px(value: f32) -> Self {
        Self::Length(Length::px(value))
    }

    pub fn to_px(
        &self,
        rel_type: Option<RelativeType>,
        rel_ctx: &RelativeContext,
        abs_ctx: &AbsoluteContext,
    ) -> f32 {
        match self {
            OffsetValue::Length(len) => len.to_px(rel_ctx, abs_ctx),
            OffsetValue::Percentage(pct) => match rel_type {
                Some(RelativeType::FontSize) => rel_ctx.parent_font_size * pct.as_fraction(),
                Some(RelativeType::ParentHeight) => rel_ctx.parent_height * pct.as_fraction(),
                Some(RelativeType::ParentWidth) => rel_ctx.parent_width * pct.as_fraction(),
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
        if value.len() != 1 {
            return Err(format!(
                "Expected exactly one ComponentValue for OffsetValue, got {}",
                value.len()
            ));
        }

        match &value[0] {
            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("calc") => {
                //Ok(Self::Calc(CalcExpression::parse(func)?))
                Err("Calc function parsing not implemented yet".to_string())
            }
            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Dimension { value, unit } => {
                    let len_unit = unit
                        .parse::<LengthUnit>()
                        .map_err(|_| format!("Invalid length unit: {}", unit))?;
                    Ok(Self::Length(Length::new(value.value as f32, len_unit)))
                }
                CssTokenKind::Percentage(pct) => {
                    Ok(Self::Percentage(Percentage::new(pct.value as f32)))
                }
                CssTokenKind::Ident(ident) if ident.eq_ignore_ascii_case("auto") => Ok(Self::Auto),
                _ => Err(format!("Invalid token for OffsetValue: {:?}", token)),
            },
            _ => Err(format!(
                "Invalid ComponentValue for OffsetValue: expected Function or Token, got {:?}",
                value[0]
            )),
        }
    }
}

impl FromStr for OffsetValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.starts_with("calc(") {
            Ok(Self::Calc(CalcExpression::parse(s)?))
        } else if let Ok(num) = s.parse::<f32>()
            && num == 0.0
        {
            Ok(Self::zero())
        } else if s.contains('%')
            && let Ok(percentage) = s.parse()
        {
            Ok(Self::Percentage(percentage))
        } else if let Ok(length) = s.parse() {
            Ok(Self::Length(length))
        } else if s.eq_ignore_ascii_case("auto") {
            Ok(Self::Auto)
        } else {
            Err(format!("Invalid OffsetValue: {}", s))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Offset {
    pub top: OffsetValue,
    pub right: OffsetValue,
    pub bottom: OffsetValue,
    pub left: OffsetValue,
}

impl Offset {
    pub fn new(
        top: OffsetValue,
        right: OffsetValue,
        bottom: OffsetValue,
        left: OffsetValue,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn zero() -> Self {
        Self {
            top: OffsetValue::zero(),
            right: OffsetValue::zero(),
            bottom: OffsetValue::zero(),
            left: OffsetValue::zero(),
        }
    }

    pub fn all(value: OffsetValue) -> Self {
        Self {
            top: value.clone(),
            right: value.clone(),
            bottom: value.clone(),
            left: value,
        }
    }

    pub fn two(vertical: OffsetValue, horizontal: OffsetValue) -> Self {
        Self {
            top: vertical.clone(),
            right: horizontal.clone(),
            bottom: vertical,
            left: horizontal,
        }
    }

    pub fn three(top: OffsetValue, horizontal: OffsetValue, bottom: OffsetValue) -> Self {
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
            2 => Ok(Offset::two(values[0].clone(), values[1].clone())),
            3 => Ok(Offset::three(
                values[0].clone(),
                values[1].clone(),
                values[2].clone(),
            )),
            4 => Ok(Offset::new(
                values[0].clone(),
                values[1].clone(),
                values[2].clone(),
                values[3].clone(),
            )),
            _ => Err(format!(
                "Invalid number of Offset values: expected 1-4, got {}",
                values.len()
            )),
        }
    }
}

impl TryFrom<&[ComponentValue]> for Offset {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let mut offset_values = Vec::new();

        for cv in value {
            match cv {
                ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("calc") => {
                    //offset_values.push(OffsetValue::Calc(CalcExpression::parse(func)?));
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| format!("Invalid length unit: {}", unit))?;
                        offset_values.push(OffsetValue::Length(Length::new(
                            value.value as f32,
                            len_unit,
                        )));
                    }
                    CssTokenKind::Percentage(pct) => {
                        offset_values
                            .push(OffsetValue::Percentage(Percentage::new(pct.value as f32)));
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

impl FromStr for Offset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        match parts.len() {
            1 => {
                let value = parts[0].parse::<OffsetValue>()?;
                Ok(Offset {
                    top: value.clone(),
                    right: value.clone(),
                    bottom: value.clone(),
                    left: value,
                })
            }
            2 => {
                let vertical = parts[0].parse::<OffsetValue>()?;
                let horizontal = parts[1].parse::<OffsetValue>()?;
                Ok(Offset {
                    top: vertical.clone(),
                    right: horizontal.clone(),
                    bottom: vertical,
                    left: horizontal,
                })
            }
            3 => {
                let top = parts[0].parse::<OffsetValue>()?;
                let horizontal = parts[1].parse::<OffsetValue>()?;
                let bottom = parts[2].parse::<OffsetValue>()?;
                Ok(Offset {
                    top,
                    right: horizontal.clone(),
                    bottom,
                    left: horizontal,
                })
            }
            4 => {
                let top = parts[0].parse::<OffsetValue>()?;
                let right = parts[1].parse::<OffsetValue>()?;
                let bottom = parts[2].parse::<OffsetValue>()?;
                let left = parts[3].parse::<OffsetValue>()?;
                Ok(Offset {
                    top,
                    right,
                    bottom,
                    left,
                })
            }
            _ => Err(format!(
                "Invalid number of Offset values: expected 1-4, got {}",
                parts.len()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use css_cssom::CssParser;

    use super::*;

    #[test]
    fn test_parse_margin() {
        assert_eq!("10px".parse(), Ok(Offset::all(OffsetValue::px(10.0))));

        assert_eq!(
            "10px 20px".parse(),
            Ok(Offset::two(OffsetValue::px(10.0), OffsetValue::px(20.0),))
        );

        assert_eq!(
            "10px 20px 30px".parse(),
            Ok(Offset::three(
                OffsetValue::px(10.0),
                OffsetValue::px(20.0),
                OffsetValue::px(30.0),
            ))
        );

        assert_eq!(
            "10px 20px 30px 40px".parse(),
            Ok(Offset::new(
                OffsetValue::px(10.0),
                OffsetValue::px(20.0),
                OffsetValue::px(30.0),
                OffsetValue::px(40.0),
            ))
        );

        assert_eq!("auto".parse(), Ok(Offset::all(OffsetValue::Auto)))
    }

    #[test]
    fn test_parse_cv() {
        let mut parser = CssParser::new(None);
        let stylesheet = parser.parse_css("* { border: 5% 2rem 20px; } ", false);
        let border = &stylesheet.rules[0].as_qualified_rule().unwrap().block.value[4];
        dbg!(&stylesheet, border);
    }
}
