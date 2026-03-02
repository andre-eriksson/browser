use std::str::FromStr;

use css_cssom::{ComponentValue, CssTokenKind};
use strum::EnumString;

use crate::{
    percentage::LengthPercentage,
    properties::gradient::{meaningful_cvs, strip_whitespace, try_parse_length_percentage},
};

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum Center {
    #[default]
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum HorizontalSide {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelativeHorizontalSide {
    Horizontal(HorizontalSide),
    Center(Center),
}

impl FromStr for RelativeHorizontalSide {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("center") {
            Ok(RelativeHorizontalSide::Center(Center::Center))
        } else if let Ok(h) = s.parse::<HorizontalSide>() {
            Ok(RelativeHorizontalSide::Horizontal(h))
        } else {
            Err(strum::ParseError::VariantNotFound)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
pub enum XSide {
    XStart,
    XEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalOrXSide {
    Horizontal(HorizontalSide),
    XSide(XSide),
}

impl FromStr for HorizontalOrXSide {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(h) = s.parse::<HorizontalSide>() {
            Ok(HorizontalOrXSide::Horizontal(h))
        } else if let Ok(x) = s.parse::<XSide>() {
            Ok(HorizontalOrXSide::XSide(x))
        } else {
            Err(strum::ParseError::VariantNotFound)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum XAxis {
    Horizontal(HorizontalSide),
    Center(Center),
    XSide(XSide),
}

impl FromStr for XAxis {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("center") {
            Ok(XAxis::Center(Center::Center))
        } else if let Ok(h) = s.parse::<HorizontalSide>() {
            Ok(XAxis::Horizontal(h))
        } else if let Ok(x) = s.parse::<XSide>() {
            Ok(XAxis::XSide(x))
        } else {
            Err(strum::ParseError::VariantNotFound)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum XAxisOrLengthPercentage {
    XAxis(XAxis),
    LengthPercentage(LengthPercentage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
pub enum YSide {
    YStart,
    YEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum VerticalSide {
    Top,
    Bottom,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelativeVerticalSide {
    Vertical(VerticalSide),
    Center(Center),
}

impl FromStr for RelativeVerticalSide {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("center") {
            Ok(RelativeVerticalSide::Center(Center::Center))
        } else if let Ok(v) = s.parse::<VerticalSide>() {
            Ok(RelativeVerticalSide::Vertical(v))
        } else {
            Err(strum::ParseError::VariantNotFound)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalOrYSide {
    Vertical(VerticalSide),
    YSide(YSide),
}

impl FromStr for VerticalOrYSide {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(v) = s.parse::<VerticalSide>() {
            Ok(VerticalOrYSide::Vertical(v))
        } else if let Ok(y) = s.parse::<YSide>() {
            Ok(VerticalOrYSide::YSide(y))
        } else {
            Err(strum::ParseError::VariantNotFound)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum YAxis {
    Vertical(VerticalSide),
    Center(Center),
    YSide(YSide),
}

impl FromStr for YAxis {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("center") {
            Ok(YAxis::Center(Center::Center))
        } else if let Ok(v) = s.parse::<VerticalSide>() {
            Ok(YAxis::Vertical(v))
        } else if let Ok(y) = s.parse::<YSide>() {
            Ok(YAxis::YSide(y))
        } else {
            Err(strum::ParseError::VariantNotFound)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum YAxisOrLengthPercentage {
    YAxis(YAxis),
    LengthPercentage(LengthPercentage),
}

#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
pub enum BlockAxis {
    BlockStart,
    BlockEnd,
}

#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
pub enum InlineAxis {
    InlineStart,
    InlineEnd,
}

#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum Side {
    Start,
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelativeAxis {
    Side(Side),
    Center(Center),
}

impl FromStr for RelativeAxis {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("center") {
            Ok(RelativeAxis::Center(Center::Center))
        } else if let Ok(side) = s.parse::<Side>() {
            Ok(RelativeAxis::Side(side))
        } else {
            Err(strum::ParseError::VariantNotFound)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PositionOne {
    Horizontal(HorizontalOrXSide),
    Center(Center),
    Vertical(VerticalOrYSide),
    BlockAxis(BlockAxis),
    InlineAxis(InlineAxis),
    LengthPercentage(LengthPercentage),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PositionTwo {
    Axis(XAxis, YAxis),
    AxisOrPercentage(XAxisOrLengthPercentage, YAxisOrLengthPercentage),
    BlockInline(BlockAxis, InlineAxis),
    Relative(RelativeAxis, RelativeAxis),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PositionThree {
    RelativeVertical(RelativeHorizontalSide, (VerticalSide, LengthPercentage)),
    RelativeHorizontal((HorizontalSide, LengthPercentage), RelativeVerticalSide),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PositionFour {
    XYPercentage((HorizontalOrXSide, LengthPercentage), (VerticalOrYSide, LengthPercentage)),
    BlockInline((BlockAxis, LengthPercentage), (InlineAxis, LengthPercentage)),
    StartEnd((Side, LengthPercentage), (Side, LengthPercentage)),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Position {
    One(PositionOne),
    Two(PositionTwo),
    Four(PositionFour),
}

#[derive(Debug, Clone)]
pub enum BackgroundPosition {
    One(PositionOne),
    Two(PositionTwo),
    Three(PositionThree),
    Four(PositionFour),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionX {
    Center(Center, Option<LengthPercentage>),
    /// At least one option must be provided, but both can be present. If both are present, the horizontal or x-side value must come before the length or percentage value.
    Relative((Option<HorizontalOrXSide>, Option<LengthPercentage>)),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionY {
    Center(Center, Option<LengthPercentage>),
    /// At least one option must be provided, but both can be present. If both are present, the vertical or y-side value must come before the length or percentage value.
    Relative((Option<VerticalOrYSide>, Option<LengthPercentage>)),
}

/// Attempt to extract the ident string from a `ComponentValue`.
fn try_ident(cv: &ComponentValue) -> Option<&str> {
    match cv {
        ComponentValue::Token(t) => match &t.kind {
            CssTokenKind::Ident(s) => Some(s.as_str()),
            _ => None,
        },
        _ => None,
    }
}

/// Check whether a `ComponentValue` is a length/percentage/zero token.
fn is_length_percentage(cv: &ComponentValue) -> bool {
    try_parse_length_percentage(cv).is_ok()
}

fn try_one_value(cv: &ComponentValue) -> Result<PositionOne, String> {
    if let Some(ident) = try_ident(cv) {
        if ident.eq_ignore_ascii_case("center") {
            return Ok(PositionOne::Center(Center::Center));
        }
        if let Ok(x) = ident.parse() {
            return Ok(PositionOne::Horizontal(x));
        }
        if let Ok(y) = ident.parse() {
            return Ok(PositionOne::Vertical(y));
        }
        if let Ok(b) = ident.parse() {
            return Ok(PositionOne::BlockAxis(b));
        }
        if let Ok(i) = ident.parse() {
            return Ok(PositionOne::InlineAxis(i));
        }
        return Err(format!("Unknown position keyword: '{}'", ident));
    }

    if is_length_percentage(cv) {
        let lp = try_parse_length_percentage(cv)?;
        return Ok(PositionOne::LengthPercentage(lp));
    }

    Err("Expected a position keyword or length/percentage".to_string())
}

fn try_two_values(a: &ComponentValue, b: &ComponentValue) -> Result<PositionTwo, String> {
    let a_ident = try_ident(a);
    let b_ident = try_ident(b);

    if let (Some(ai), Some(bi)) = (a_ident, b_ident) {
        if let (Ok(ba), Ok(ia)) = (ai.parse::<BlockAxis>(), bi.parse::<InlineAxis>()) {
            return Ok(PositionTwo::BlockInline(ba, ia));
        }

        if let (Ok(ra), Ok(rb)) = (ai.parse(), bi.parse())
            && ai.parse::<XAxis>().is_err()
            && ai.parse::<YAxis>().is_err()
        {
            return Ok(PositionTwo::Relative(ra, rb));
        }

        if let (Ok(x), Ok(y)) = (ai.parse(), bi.parse()) {
            return Ok(PositionTwo::Axis(x, y));
        }

        if let (Ok(y), Ok(x)) = (ai.parse(), bi.parse()) {
            return Ok(PositionTwo::Axis(x, y));
        }
    }

    let x_or_lp = if let Some(ident) = a_ident {
        if let Ok(x) = ident.parse() {
            XAxisOrLengthPercentage::XAxis(x)
        } else {
            return Err(format!("Invalid x-axis position: '{}'", ident));
        }
    } else if is_length_percentage(a) {
        XAxisOrLengthPercentage::LengthPercentage(try_parse_length_percentage(a)?)
    } else {
        return Err("Expected position keyword or length/percentage for x component".to_string());
    };

    let y_or_lp = if let Some(ident) = b_ident {
        if let Ok(y) = ident.parse() {
            YAxisOrLengthPercentage::YAxis(y)
        } else {
            return Err(format!("Invalid y-axis position: '{}'", ident));
        }
    } else if is_length_percentage(b) {
        YAxisOrLengthPercentage::LengthPercentage(try_parse_length_percentage(b)?)
    } else {
        return Err("Expected position keyword or length/percentage for y component".to_string());
    };

    Ok(PositionTwo::AxisOrPercentage(x_or_lp, y_or_lp))
}

fn try_three_values(a: &ComponentValue, b: &ComponentValue, c: &ComponentValue) -> Result<PositionThree, String> {
    let a_ident = try_ident(a).ok_or("Expected keyword for first of 3-value position")?;
    let c_ident = try_ident(c).ok_or("Expected keyword for third of 3-value position")?;
    let b_lp = try_parse_length_percentage(b)?;

    if let (Ok(rh), Ok(v)) = (a_ident.parse(), c_ident.parse()) {
        return Ok(PositionThree::RelativeVertical(rh, (v, b_lp)));
    }
    if let (Ok(h), Ok(rv)) = (a_ident.parse(), c_ident.parse()) {
        return Ok(PositionThree::RelativeHorizontal((h, b_lp), rv));
    }

    Err(format!("Invalid 3-value position: '{}' and '{}' are not a valid axis pair", a_ident, c_ident))
}

fn try_four_values(
    a: &ComponentValue,
    b: &ComponentValue,
    c: &ComponentValue,
    d: &ComponentValue,
) -> Result<PositionFour, String> {
    let a_ident = try_ident(a).ok_or("Expected keyword for first of 4-value position")?;
    let c_ident = try_ident(c).ok_or("Expected keyword for third of 4-value position")?;
    let b_lp = try_parse_length_percentage(b)?;
    let d_lp = try_parse_length_percentage(d)?;

    if let (Ok(ba), Ok(ia)) = (a_ident.parse::<BlockAxis>(), c_ident.parse::<InlineAxis>()) {
        return Ok(PositionFour::BlockInline((ba, b_lp), (ia, d_lp)));
    }
    if let (Ok(ia), Ok(ba)) = (a_ident.parse::<InlineAxis>(), c_ident.parse::<BlockAxis>()) {
        return Ok(PositionFour::BlockInline((ba, d_lp), (ia, b_lp)));
    }

    if let (Ok(s1), Ok(s2)) = (a_ident.parse::<Side>(), c_ident.parse::<Side>()) {
        return Ok(PositionFour::StartEnd((s1, b_lp), (s2, d_lp)));
    }

    if let (Ok(h), Ok(v)) = (a_ident.parse(), c_ident.parse()) {
        return Ok(PositionFour::XYPercentage((h, b_lp), (v, d_lp)));
    }

    if let (Ok(v), Ok(h)) = (a_ident.parse(), c_ident.parse()) {
        return Ok(PositionFour::XYPercentage((h, d_lp), (v, b_lp)));
    }

    Err(format!("Invalid 4-value position: '{}' and '{}' are not a valid axis pair", a_ident, c_ident))
}

impl TryFrom<&[ComponentValue]> for Position {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let stripped = strip_whitespace(value);
        if stripped.is_empty() {
            return Err("Empty position".to_string());
        }

        let tokens = meaningful_cvs(stripped);

        match tokens.len() {
            1 => {
                let one = try_one_value(tokens[0])?;
                Ok(Position::One(one))
            }
            2 => {
                let two = try_two_values(tokens[0], tokens[1])?;
                Ok(Position::Two(two))
            }
            3 => Err("3-value <position> syntax is not supported for <position>. Did you mean <bg-position>?".into()),
            4 => {
                let four = try_four_values(tokens[0], tokens[1], tokens[2], tokens[3])?;
                Ok(Position::Four(four))
            }
            n => Err(format!("Too many tokens for <position> (expected 1-4, got {})", n)),
        }
    }
}

impl TryFrom<&[ComponentValue]> for BackgroundPosition {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let stripped = strip_whitespace(value);
        if stripped.is_empty() {
            return Err("Empty background-position".to_string());
        }

        let tokens = meaningful_cvs(stripped);

        match tokens.len() {
            1 => Ok(BackgroundPosition::One(try_one_value(tokens[0])?)),
            2 => Ok(BackgroundPosition::Two(try_two_values(tokens[0], tokens[1])?)),
            3 => Ok(BackgroundPosition::Three(try_three_values(tokens[0], tokens[1], tokens[2])?)),
            4 => Ok(BackgroundPosition::Four(try_four_values(tokens[0], tokens[1], tokens[2], tokens[3])?)),
            n => Err(format!("Too many tokens for <background-position> (expected 1-4, got {})", n)),
        }
    }
}
