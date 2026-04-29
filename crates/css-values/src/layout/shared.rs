use std::str::FromStr;

use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};
use strum::EnumString;

use crate::{
    CSSParsable,
    calc::{CalcDomain, CalcExpression, is_math_function},
    error::CssValueError,
    numeric::Percentage,
    position::HorizontalSide,
    quantity::Length,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Gap {
    #[default]
    Normal,
    Length(Length),
    Percentage(Percentage),
    Calc(CalcExpression),
}

impl CSSParsable for Gap {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        if let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Function(func) => {
                    if is_math_function(&func.name) {
                        let expr = CalcExpression::parse(&func.name, &func.value)?;
                        let domain = expr.resolve_domain()?;

                        if !matches!(domain, CalcDomain::Length | CalcDomain::Percentage) {
                            return Err(CssValueError::InvalidCalcDomain {
                                expected: vec![CalcDomain::Length, CalcDomain::Percentage],
                                found: domain,
                            });
                        }

                        Ok(Self::Calc(expr))
                    } else {
                        Err(CssValueError::InvalidFunction(func.name.clone()))
                    }
                }
                ComponentValue::Token(token) => {
                    if let Ok(length) = Length::try_from(token) {
                        Ok(Self::Length(length))
                    } else if let Ok(percentage) = Percentage::try_from(token) {
                        Ok(Self::Percentage(percentage))
                    } else if let CssTokenKind::Ident(ident) = &token.kind
                        && ident.eq_ignore_ascii_case("normal")
                    {
                        Ok(Self::Normal)
                    } else {
                        Err(CssValueError::InvalidToken(token.kind.clone()))
                    }
                }
                cvs @ ComponentValue::SimpleBlock(_) => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            }
        } else {
            Err(CssValueError::UnexpectedEndOfInput)
        }
    }
}

/// # Syntax
/// ```text
/// <content-distribution> =
///  space-between  |
///  space-around   |
///  space-evenly   |
///  stretch
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
pub enum ContentDistribution {
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Stretch,
}

/// # Syntax
/// ```text
/// <overflow-position> =
///  unsafe  |
///  safe
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum OverflowPosition {
    Safe,
    Unsafe,
}

/// # Syntax
/// ```text
/// <content-position> =
///   center      |
///   start       |
///   end         |
///   flex-start  |
///   flex-end
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
pub enum ContentPosition {
    Center,
    Start,
    End,
    FlexStart,
    FlexEnd,
}

/// # Syntax
/// ```text
/// <self-position> =
///   center      |
///   start       |
///   end         |
///   self-start  |
///   self-end    |
///   flex-start  |
///   flex-end
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
pub enum SelfPosition {
    Center,
    Start,
    End,
    SelfStart,
    SelfEnd,
    FlexStart,
    FlexEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum BaselinePosition {
    First,
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentAlignment {
    ContentPosition(ContentPosition),
    HorizontalSide(HorizontalSide),
}

impl FromStr for ContentAlignment {
    type Err = CssValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(position) = ContentPosition::from_str(s) {
            Ok(Self::ContentPosition(position))
        } else if let Ok(side) = HorizontalSide::from_str(s) {
            Ok(Self::HorizontalSide(side))
        } else {
            Err(CssValueError::InvalidValue(format!("Invalid content alignment: {}", s)))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemsAlignment {
    SelfPosition(SelfPosition),
    HorizontalSide(HorizontalSide),
}

impl FromStr for ItemsAlignment {
    type Err = CssValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(position) = SelfPosition::from_str(s) {
            Ok(Self::SelfPosition(position))
        } else if let Ok(side) = HorizontalSide::from_str(s) {
            Ok(Self::HorizontalSide(side))
        } else {
            Err(CssValueError::InvalidValue(format!("Invalid items alignment: {}", s)))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignSelfAlignment {
    Normal,
    SelfPosition(SelfPosition),
}

impl FromStr for AlignSelfAlignment {
    type Err = CssValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(position) = SelfPosition::from_str(s) {
            Ok(Self::SelfPosition(position))
        } else if s.eq_ignore_ascii_case("normal") {
            Ok(Self::Normal)
        } else {
            Err(CssValueError::InvalidValue(format!("Invalid align-self alignment: {}", s)))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifySelfAlignment {
    Normal,
    SelfPosition(SelfPosition),
    HorizontalSide(HorizontalSide),
}

impl FromStr for JustifySelfAlignment {
    type Err = CssValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(position) = SelfPosition::from_str(s) {
            Ok(Self::SelfPosition(position))
        } else if let Ok(side) = HorizontalSide::from_str(s) {
            Ok(Self::HorizontalSide(side))
        } else if s.eq_ignore_ascii_case("normal") {
            Ok(Self::Normal)
        } else {
            Err(CssValueError::InvalidValue(format!("Invalid justify-self alignment: {}", s)))
        }
    }
}
