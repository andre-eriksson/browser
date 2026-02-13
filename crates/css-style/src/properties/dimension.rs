use css_cssom::{ComponentValue, CssTokenKind};

use crate::{
    calculate::CalcExpression,
    length::LengthUnit,
    primitives::{length::Length, percentage::Percentage},
    properties::{AbsoluteContext, RelativeContext, RelativeType},
};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Dimension {
    Percentage(Percentage),
    Length(Length),
    Calc(CalcExpression),
    #[default]
    Auto,
    MaxContent,
    MinContent,
    FitContent(Option<Length>),
    Stretch,
}

impl Dimension {
    pub fn px(value: f32) -> Self {
        Self::Length(Length::px(value))
    }

    pub fn to_px(
        &self,
        rel_type: RelativeType,
        rel_ctx: &RelativeContext,
        abs_ctx: &AbsoluteContext,
    ) -> f32 {
        match self {
            Dimension::Length(l) => l.to_px(rel_ctx, abs_ctx),
            Dimension::MaxContent => 0.0,
            Dimension::MinContent => 0.0,
            Dimension::FitContent(_) => 0.0,
            Dimension::Stretch => 0.0,
            Dimension::Auto => 0.0,
            Dimension::Calc(calc) => calc.to_px(Some(rel_type), rel_ctx, abs_ctx),
            Dimension::Percentage(p) => match rel_type {
                RelativeType::FontSize => rel_ctx.parent_font_size * p.as_fraction(),
                RelativeType::ParentHeight => rel_ctx.parent_height * p.as_fraction(),
                RelativeType::ParentWidth => rel_ctx.parent_width * p.as_fraction(),
                RelativeType::RootFontSize => abs_ctx.root_font_size * p.as_fraction(),
                RelativeType::ViewportHeight => abs_ctx.viewport_height * p.as_fraction(),
                RelativeType::ViewportWidth => abs_ctx.viewport_width * p.as_fraction(),
            },
        }
    }
}

impl TryFrom<&[ComponentValue]> for Dimension {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Function(func) => {
                    if func.name.eq_ignore_ascii_case("calc") {
                        // return Ok(Dimension::Calc(CalcExpression::parse_function(func)?));
                    }
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("auto") {
                            return Ok(Dimension::Auto);
                        } else if ident.eq_ignore_ascii_case("max-content") {
                            return Ok(Dimension::MaxContent);
                        } else if ident.eq_ignore_ascii_case("min-content") {
                            return Ok(Dimension::MinContent);
                        } else if ident.eq_ignore_ascii_case("fit-content") {
                            return Ok(Dimension::FitContent(None)); // TODO: Fix?
                        } else if ident.eq_ignore_ascii_case("stretch") {
                            return Ok(Dimension::Stretch);
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| format!("Invalid length unit: {}", unit))?;
                        return Ok(Dimension::Length(Length::new(value.value as f32, len_unit)));
                    }
                    CssTokenKind::Percentage(pct) => {
                        return Ok(Dimension::Percentage(Percentage::new(pct.value as f32)));
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Err("No valid Dimension found in component values".to_string())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum MaxDimension {
    Length(Length),
    Percentage(Percentage),
    Calc(CalcExpression),
    #[default]
    None,
    MaxContent,
    MinContent,
    FitContent(Option<Length>),
    Stretch,
}

impl TryFrom<&[ComponentValue]> for MaxDimension {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Function(func) => {
                    if func.name.eq_ignore_ascii_case("calc") {
                        // return Ok(MaxDimension::Calc(CalcExpression::parse_function(func)?));
                    }
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("none") {
                            return Ok(MaxDimension::None);
                        } else if ident.eq_ignore_ascii_case("max-content") {
                            return Ok(MaxDimension::MaxContent);
                        } else if ident.eq_ignore_ascii_case("min-content") {
                            return Ok(MaxDimension::MinContent);
                        } else if ident.eq_ignore_ascii_case("fit-content") {
                            return Ok(MaxDimension::FitContent(None)); // TODO: Fix?
                        } else if ident.eq_ignore_ascii_case("stretch") {
                            return Ok(MaxDimension::Stretch);
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| format!("Invalid length unit: {}", unit))?;
                        return Ok(MaxDimension::Length(Length::new(
                            value.value as f32,
                            len_unit,
                        )));
                    }
                    CssTokenKind::Percentage(pct) => {
                        return Ok(MaxDimension::Percentage(Percentage::new(pct.value as f32)));
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Err("No valid MaxDimension found in component values".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
