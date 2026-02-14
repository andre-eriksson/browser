use css_cssom::{ComponentValue, CssTokenKind};
use strum::EnumString;

use crate::{
    ComputedStyle, RelativeType,
    calculate::CalcExpression,
    length::LengthUnit,
    primitives::{length::Length, percentage::Percentage},
    properties::{AbsoluteContext, RelativeContext},
};

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum WritingMode {
    #[default]
    HorizontalTb,
    VerticalRl,
    VerticalLr,
    SidewaysRl,
    SidewaysLr,
}

impl TryFrom<&[ComponentValue]> for WritingMode {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Token(token) => {
                    if let css_cssom::CssTokenKind::Ident(ident) = &token.kind
                        && let Ok(mode) = ident.parse()
                    {
                        return Ok(mode);
                    }
                }
                _ => continue,
            }
        }
        Err("No valid writing-mode value found".to_string())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum TextAlign {
    Start,
    End,
    #[default]
    Left,
    Right,
    Center,
    Justify,
    MatchParent,
}

impl TryFrom<&[ComponentValue]> for TextAlign {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Token(token) => {
                    if let css_cssom::CssTokenKind::Ident(ident) = &token.kind
                        && let Ok(align) = ident.parse()
                    {
                        return Ok(align);
                    }
                }
                _ => continue,
            }
        }
        Err("No valid text-align value found".to_string())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum Whitespace {
    #[default]
    Normal,
    Pre,
    PreWrap,
    PreLine,

    // Shorthands, TODO: expand these into their full equivalents when parsing
    Nowrap,
    Wrap,
    BreakSpaces,
    Collapse,
}

impl TryFrom<&[ComponentValue]> for Whitespace {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Token(token) => {
                    if let css_cssom::CssTokenKind::Ident(ident) = &token.kind
                        && let Ok(ws) = ident.parse()
                    {
                        return Ok(ws);
                    }
                }
                _ => continue,
            }
        }
        Err("No valid whitespace value found".to_string())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum LineHeight {
    #[default]
    Normal,
    Number(f32),
    Length(Length),
    Percentage(Percentage),
    Calc(CalcExpression),
}

impl LineHeight {
    pub fn px(value: f32) -> Self {
        LineHeight::Length(Length::px(value))
    }

    pub fn to_px(&self, abs_ctx: &AbsoluteContext, font_size_px: f32) -> f32 {
        let rel_ctx = RelativeContext {
            parent: ComputedStyle {
                font_size: font_size_px,
                ..Default::default()
            }
            .into(),
        };

        match self {
            LineHeight::Normal => font_size_px * 1.2,
            LineHeight::Number(num) => font_size_px * num,
            LineHeight::Length(len) => len.to_px(&rel_ctx, abs_ctx),
            LineHeight::Percentage(pct) => pct.as_fraction() * font_size_px,
            LineHeight::Calc(calc) => calc.to_px(Some(RelativeType::FontSize), &rel_ctx, abs_ctx),
        }
    }
}

impl TryFrom<&[ComponentValue]> for LineHeight {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("calc") => {
                    return Ok(LineHeight::Calc(CalcExpression::parse(
                        func.value.as_slice(),
                    )?));
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) if ident.eq_ignore_ascii_case("normal") => {
                        return Ok(LineHeight::Normal);
                    }
                    CssTokenKind::Number(num) => {
                        return Ok(LineHeight::Number(num.value as f32));
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| format!("Invalid length unit: {}", unit))?;
                        return Ok(LineHeight::Length(Length::new(
                            value.value as f32,
                            len_unit,
                        )));
                    }
                    CssTokenKind::Percentage(pct) => {
                        return Ok(LineHeight::Percentage(Percentage::new(pct.value as f32)));
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Err("No valid line-height value found".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_align() {
        assert_eq!("start".parse(), Ok(TextAlign::Start));
        assert_eq!("end".parse(), Ok(TextAlign::End));
        assert_eq!("left".parse(), Ok(TextAlign::Left));
        assert_eq!("right".parse(), Ok(TextAlign::Right));
        assert_eq!("center".parse(), Ok(TextAlign::Center));
        assert_eq!("justify".parse(), Ok(TextAlign::Justify));
        assert_eq!("match-parent".parse(), Ok(TextAlign::MatchParent));
        assert!("unknown".parse::<TextAlign>().is_err());
    }
}
