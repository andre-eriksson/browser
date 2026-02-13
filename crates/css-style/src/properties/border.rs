use css_cssom::{ComponentValue, CssTokenKind};
use strum::EnumString;

use crate::{
    calculate::CalcExpression,
    length::LengthUnit,
    primitives::length::Length,
    properties::{AbsoluteContext, RelativeContext},
};

#[derive(Debug, Clone, PartialEq)]
pub enum BorderWidth {
    Length(Length),
    Calc(CalcExpression),
    Thin,
    Medium,
    Thick,
}

impl Default for BorderWidth {
    fn default() -> Self {
        BorderWidth::Length(Length::px(0.0))
    }
}

impl TryFrom<&[ComponentValue]> for BorderWidth {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("thin") {
                            return Ok(BorderWidth::Thin);
                        } else if ident.eq_ignore_ascii_case("medium") {
                            return Ok(BorderWidth::Medium);
                        } else if ident.eq_ignore_ascii_case("thick") {
                            return Ok(BorderWidth::Thick);
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| format!("Invalid length unit: {}", unit))?;
                        return Ok(BorderWidth::Length(Length::new(
                            value.value as f32,
                            len_unit,
                        )));
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Err("No valid BorderWidthValue found".to_string())
    }
}

impl BorderWidth {
    pub fn px(value: f32) -> Self {
        BorderWidth::Length(Length::px(value))
    }

    pub fn to_px(&self, rel_ctx: &RelativeContext, abs_ctx: &AbsoluteContext) -> f32 {
        match self {
            BorderWidth::Length(len) => len.to_px(rel_ctx, abs_ctx),
            BorderWidth::Calc(calc) => calc.to_px(None, rel_ctx, abs_ctx),
            BorderWidth::Thin => 1.0,
            BorderWidth::Medium => 3.0,
            BorderWidth::Thick => 5.0,
        }
    }
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum BorderStyle {
    #[default]
    None,
    Hidden,
    Dotted,
    Dashed,
    Solid,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

impl TryFrom<&[ComponentValue]> for BorderStyle {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        if let Some(cv) = value.iter().next()
            && let ComponentValue::Token(token) = cv
            && let CssTokenKind::Ident(ident) = &token.kind
        {
            ident
                .parse()
                .map_err(|_| format!("Invalid named color: '{}'", ident))
        } else {
            Err(format!(
                "No valid named color token found in component values: {:?}",
                value
            ))
        }
    }
}
