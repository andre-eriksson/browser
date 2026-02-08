use std::str::FromStr;

use crate::{
    calculate::CalcExpression,
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
            Dimension::Calc(calc) => calc.to_px(rel_type, rel_ctx, abs_ctx),
            Dimension::Percentage(p) => match rel_type {
                RelativeType::FontSize => rel_ctx.font_size * p.as_fraction(),
                RelativeType::ParentHeight => rel_ctx.parent_height * p.as_fraction(),
                RelativeType::ParentWidth => rel_ctx.parent_width * p.as_fraction(),
                RelativeType::RootFontSize => abs_ctx.root_font_size * p.as_fraction(),
                RelativeType::ViewportHeight => abs_ctx.viewport_height * p.as_fraction(),
                RelativeType::ViewportWidth => abs_ctx.viewport_width * p.as_fraction(),
            },
        }
    }
}

impl FromStr for Dimension {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.starts_with("calc(") {
            Ok(Self::Calc(CalcExpression::parse(s)?))
        } else if let Ok(num) = s.parse::<f32>()
            && num == 0.0
        {
            Ok(Self::Length(Length::px(0.0)))
        } else if s.contains('%')
            && let Ok(percentage) = s.parse()
        {
            Ok(Self::Percentage(percentage))
        } else if let Ok(length) = s.parse() {
            Ok(Self::Length(length))
        } else if s.eq_ignore_ascii_case("auto") {
            Ok(Self::Auto)
        } else if s.eq_ignore_ascii_case("max-content") {
            Ok(Self::MaxContent)
        } else if s.eq_ignore_ascii_case("min-content") {
            Ok(Self::MinContent)
        } else if s.eq_ignore_ascii_case("fit-content") {
            Ok(Self::FitContent(None)) // TODO: Fix?
        } else if s.eq_ignore_ascii_case("stretch") {
            Ok(Self::Stretch)
        } else {
            Err(format!("Invalid Dimension value: {}", s))
        }
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

impl FromStr for MaxDimension {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.starts_with("calc(") {
            Ok(Self::Calc(CalcExpression::parse(s)?))
        } else if s.contains('%')
            && let Ok(percentage) = s.parse()
        {
            Ok(Self::Percentage(percentage))
        } else if let Ok(length) = s.parse() {
            Ok(Self::Length(length))
        } else if s.eq_ignore_ascii_case("none") {
            Ok(Self::None)
        } else if s.eq_ignore_ascii_case("max-content") {
            Ok(Self::MaxContent)
        } else if s.eq_ignore_ascii_case("min-content") {
            Ok(Self::MinContent)
        } else if s.eq_ignore_ascii_case("fit-content") {
            Ok(Self::FitContent(None)) // TODO: Fix?
        } else if s.eq_ignore_ascii_case("stretch") {
            Ok(Self::Stretch)
        } else {
            Err(format!("Invalid MaxDimension value: {}", s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_width() {
        assert_eq!(
            "50%".parse(),
            Ok(Dimension::Percentage(Percentage::new(50.0)))
        );
        assert_eq!("auto".parse(), Ok(Dimension::Auto));
        assert_eq!("max-content".parse(), Ok(Dimension::MaxContent));
        assert_eq!("min-content".parse(), Ok(Dimension::MinContent));
        assert_eq!("fit-content".parse(), Ok(Dimension::FitContent(None)));
        assert_eq!("stretch".parse(), Ok(Dimension::Stretch));
    }
}
