use std::str::FromStr;

use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};
use strum::EnumString;

use crate::{
    length::{Length, LengthUnit},
    percentage::{LengthPercentage, Percentage},
    properties::CSSParsable,
};

/// A classified token extracted from the stream. Owns its data so the stream
/// borrow is released immediately, allowing successive reads.
enum PosToken {
    Ident(String),
    LengthPercentage(LengthPercentage),
}

/// Consume the next non-whitespace token from `stream` and classify it as
/// either an ident or a length/percentage. Returns `None` if the stream is
/// exhausted or the token is neither.
fn next_pos_token(stream: &mut ComponentValueStream) -> Option<PosToken> {
    match stream.next_non_whitespace()? {
        ComponentValue::Token(t) => match &t.kind {
            CssTokenKind::Ident(s) => Some(PosToken::Ident(s.clone())),
            CssTokenKind::Dimension { value, unit } => {
                let len_unit = unit.parse::<LengthUnit>().ok()?;
                Some(PosToken::LengthPercentage(LengthPercentage::Length(Length::new(value.to_f64() as f32, len_unit))))
            }
            CssTokenKind::Percentage(pct) => {
                Some(PosToken::LengthPercentage(LengthPercentage::Percentage(Percentage::new(pct.to_f64() as f32))))
            }
            CssTokenKind::Number(n) if n.to_f64() == 0.0 => {
                Some(PosToken::LengthPercentage(LengthPercentage::Length(Length::new(0.0, LengthUnit::Px))))
            }
            _ => None,
        },
        _ => None,
    }
}

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

impl CSSParsable for PositionOne {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        while let Some(cv) = stream.next_cv() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("center") {
                            return Ok(PositionOne::Center(Center::Center));
                        } else if let Ok(x) = ident.parse() {
                            return Ok(PositionOne::Horizontal(x));
                        } else if let Ok(y) = ident.parse() {
                            return Ok(PositionOne::Vertical(y));
                        } else if let Ok(b) = ident.parse() {
                            return Ok(PositionOne::BlockAxis(b));
                        } else if let Ok(i) = ident.parse() {
                            return Ok(PositionOne::InlineAxis(i));
                        } else {
                            return Err(format!("Unknown position keyword: '{}'", ident));
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| "Invalid length unit".to_string())?;
                        let len = Length::new(value.to_f64() as f32, len_unit);
                        return Ok(PositionOne::LengthPercentage(LengthPercentage::Length(len)));
                    }
                    CssTokenKind::Percentage(pct) => {
                        let percentage = Percentage::new(pct.to_f64() as f32);
                        return Ok(PositionOne::LengthPercentage(LengthPercentage::Percentage(percentage)));
                    }
                    _ => continue,
                },
                _ => return Err("Expected a token for position".to_string()),
            }
        }

        Err("Expected a position keyword or length/percentage".to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PositionTwo {
    /// X Axis and Y Axis can be in any order, but both must be present and valid axes.
    Axis(XAxis, YAxis),

    /// X Axis must be followed by Y Axis, but each can be either an axis keyword or a length/percentage.
    AxisOrPercentage(XAxisOrLengthPercentage, YAxisOrLengthPercentage),

    /// Block axis and inline axis can be in either order, but both must be present and valid axes.
    BlockInline(BlockAxis, InlineAxis),

    /// Two relative-axis keywords, both must be present and valid relative-axis keywords.
    Relative(RelativeAxis, RelativeAxis),
}

impl PositionTwo {
    fn try_block_inline(stream: &mut ComponentValueStream) -> Result<Self, String> {
        let checkpoint = stream.checkpoint();
        let ai = match next_pos_token(stream) {
            Some(PosToken::Ident(s)) => s,
            _ => {
                stream.restore(checkpoint);
                return Err("Expected block-axis keyword".into());
            }
        };
        let bi = match next_pos_token(stream) {
            Some(PosToken::Ident(s)) => s,
            _ => {
                stream.restore(checkpoint);
                return Err("Expected inline-axis keyword".into());
            }
        };
        if let (Ok(ba), Ok(ia)) = (ai.parse::<BlockAxis>(), bi.parse::<InlineAxis>()) {
            return Ok(PositionTwo::BlockInline(ba, ia));
        }
        if let (Ok(ia), Ok(ba)) = (ai.parse::<InlineAxis>(), bi.parse::<BlockAxis>()) {
            return Ok(PositionTwo::BlockInline(ba, ia));
        }
        stream.restore(checkpoint);
        Err("Not a block-inline pair".into())
    }

    fn try_relative(stream: &mut ComponentValueStream) -> Result<Self, String> {
        let checkpoint = stream.checkpoint();
        let ai = match next_pos_token(stream) {
            Some(PosToken::Ident(s)) => s,
            _ => {
                stream.restore(checkpoint);
                return Err("Expected relative-axis keyword".into());
            }
        };
        if ai.parse::<XAxis>().is_ok() || ai.parse::<YAxis>().is_ok() {
            stream.restore(checkpoint);
            return Err("Not a relative pair".into());
        }
        let ra = match ai.parse::<RelativeAxis>() {
            Ok(r) => r,
            Err(_) => {
                stream.restore(checkpoint);
                return Err("Not a relative-axis keyword".into());
            }
        };
        let bi = match next_pos_token(stream) {
            Some(PosToken::Ident(s)) => s,
            _ => {
                stream.restore(checkpoint);
                return Err("Expected second relative-axis keyword".into());
            }
        };
        if let Ok(rb) = bi.parse::<RelativeAxis>() {
            return Ok(PositionTwo::Relative(ra, rb));
        }
        stream.restore(checkpoint);
        Err("Not a relative pair".into())
    }

    fn try_axis(stream: &mut ComponentValueStream) -> Result<Self, String> {
        let checkpoint = stream.checkpoint();
        let ai = match next_pos_token(stream) {
            Some(PosToken::Ident(s)) => s,
            _ => {
                stream.restore(checkpoint);
                return Err("Expected axis keyword".into());
            }
        };
        let bi = match next_pos_token(stream) {
            Some(PosToken::Ident(s)) => s,
            _ => {
                stream.restore(checkpoint);
                return Err("Expected axis keyword".into());
            }
        };
        if let (Ok(x), Ok(y)) = (ai.parse::<XAxis>(), bi.parse::<YAxis>()) {
            return Ok(PositionTwo::Axis(x, y));
        }
        if let (Ok(y), Ok(x)) = (ai.parse::<YAxis>(), bi.parse::<XAxis>()) {
            return Ok(PositionTwo::Axis(x, y));
        }
        stream.restore(checkpoint);
        Err("Not an axis pair".into())
    }

    fn try_axis_or_percentage(stream: &mut ComponentValueStream) -> Result<Self, String> {
        let checkpoint = stream.checkpoint();

        let x_or_lp = match next_pos_token(stream) {
            Some(PosToken::Ident(s)) => match s.parse::<XAxis>() {
                Ok(x) => XAxisOrLengthPercentage::XAxis(x),
                Err(_) => {
                    stream.restore(checkpoint);
                    return Err(format!("Invalid x-axis position: '{}'", s));
                }
            },
            Some(PosToken::LengthPercentage(lp)) => XAxisOrLengthPercentage::LengthPercentage(lp),
            None => {
                stream.restore(checkpoint);
                return Err("Expected position keyword or length/percentage for x component".into());
            }
        };

        let y_or_lp = match next_pos_token(stream) {
            Some(PosToken::Ident(s)) => match s.parse::<YAxis>() {
                Ok(y) => YAxisOrLengthPercentage::YAxis(y),
                Err(_) => {
                    stream.restore(checkpoint);
                    return Err(format!("Invalid y-axis position: '{}'", s));
                }
            },
            Some(PosToken::LengthPercentage(lp)) => YAxisOrLengthPercentage::LengthPercentage(lp),
            None => {
                stream.restore(checkpoint);
                return Err("Expected position keyword or length/percentage for y component".into());
            }
        };

        Ok(PositionTwo::AxisOrPercentage(x_or_lp, y_or_lp))
    }
}

impl CSSParsable for PositionTwo {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        if let Ok(v) = Self::try_block_inline(stream) {
            Ok(v)
        } else if let Ok(v) = Self::try_relative(stream) {
            Ok(v)
        } else if let Ok(v) = Self::try_axis(stream) {
            Ok(v)
        } else {
            Self::try_axis_or_percentage(stream)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PositionThree {
    RelativeVertical(RelativeHorizontalSide, (VerticalSide, LengthPercentage)),
    RelativeHorizontal((HorizontalSide, LengthPercentage), RelativeVerticalSide),
}

impl PositionThree {
    /// Try: keyword length-percentage keyword (e.g. `left 20px center`)
    fn try_keyword_lp_keyword(stream: &mut ComponentValueStream) -> Result<Self, String> {
        let checkpoint = stream.checkpoint();
        let ai = match next_pos_token(stream) {
            Some(PosToken::Ident(s)) => s,
            _ => {
                stream.restore(checkpoint);
                return Err("Expected keyword".into());
            }
        };
        let lp = match next_pos_token(stream) {
            Some(PosToken::LengthPercentage(lp)) => lp,
            _ => {
                stream.restore(checkpoint);
                return Err("Expected length/percentage".into());
            }
        };
        let ci = match next_pos_token(stream) {
            Some(PosToken::Ident(s)) => s,
            _ => {
                stream.restore(checkpoint);
                return Err("Expected keyword".into());
            }
        };
        if let (Ok(h), Ok(rv)) = (ai.parse::<HorizontalSide>(), ci.parse::<RelativeVerticalSide>()) {
            return Ok(PositionThree::RelativeHorizontal((h, lp), rv));
        }
        if let (Ok(v), Ok(rh)) = (ai.parse::<VerticalSide>(), ci.parse::<RelativeHorizontalSide>()) {
            return Ok(PositionThree::RelativeVertical(rh, (v, lp)));
        }
        stream.restore(checkpoint);
        Err("Not a keyword-lp-keyword 3-value position".into())
    }

    /// Try: keyword keyword length-percentage (e.g. `center bottom 20px`)
    fn try_keyword_keyword_lp(stream: &mut ComponentValueStream) -> Result<Self, String> {
        let checkpoint = stream.checkpoint();
        let ai = match next_pos_token(stream) {
            Some(PosToken::Ident(s)) => s,
            _ => {
                stream.restore(checkpoint);
                return Err("Expected keyword".into());
            }
        };
        let bi = match next_pos_token(stream) {
            Some(PosToken::Ident(s)) => s,
            _ => {
                stream.restore(checkpoint);
                return Err("Expected keyword".into());
            }
        };
        let lp = match next_pos_token(stream) {
            Some(PosToken::LengthPercentage(lp)) => lp,
            _ => {
                stream.restore(checkpoint);
                return Err("Expected length/percentage".into());
            }
        };
        if let (Ok(rh), Ok(v)) = (ai.parse::<RelativeHorizontalSide>(), bi.parse::<VerticalSide>()) {
            return Ok(PositionThree::RelativeVertical(rh, (v, lp)));
        }
        if let (Ok(rv), Ok(h)) = (ai.parse::<RelativeVerticalSide>(), bi.parse::<HorizontalSide>()) {
            return Ok(PositionThree::RelativeHorizontal((h, lp), rv));
        }
        stream.restore(checkpoint);
        Err("Not a keyword-keyword-lp 3-value position".into())
    }
}

impl CSSParsable for PositionThree {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        if let Ok(v) = Self::try_keyword_lp_keyword(stream) {
            Ok(v)
        } else if let Ok(v) = Self::try_keyword_keyword_lp(stream) {
            Ok(v)
        } else {
            Err("Invalid 3-value position: expected two keywords and one length/percentage".into())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PositionFour {
    XYPercentage((HorizontalOrXSide, LengthPercentage), (VerticalOrYSide, LengthPercentage)),
    BlockInline((BlockAxis, LengthPercentage), (InlineAxis, LengthPercentage)),
    StartEnd((Side, LengthPercentage), (Side, LengthPercentage)),
}

impl PositionFour {
    /// Parse the common 4-value shape: ident lp ident lp, then classify.
    fn parse_ident_lp_ident_lp(stream: &mut ComponentValueStream) -> Result<Self, String> {
        let checkpoint = stream.checkpoint();

        let ai = match next_pos_token(stream) {
            Some(PosToken::Ident(s)) => s,
            _ => {
                stream.restore(checkpoint);
                return Err("Expected keyword for first of 4-value position".into());
            }
        };
        let b_lp = match next_pos_token(stream) {
            Some(PosToken::LengthPercentage(lp)) => lp,
            _ => {
                stream.restore(checkpoint);
                return Err("Expected length/percentage for second of 4-value position".into());
            }
        };
        let ci = match next_pos_token(stream) {
            Some(PosToken::Ident(s)) => s,
            _ => {
                stream.restore(checkpoint);
                return Err("Expected keyword for third of 4-value position".into());
            }
        };
        let d_lp = match next_pos_token(stream) {
            Some(PosToken::LengthPercentage(lp)) => lp,
            _ => {
                stream.restore(checkpoint);
                return Err("Expected length/percentage for fourth of 4-value position".into());
            }
        };

        if let (Ok(ba), Ok(ia)) = (ai.parse::<BlockAxis>(), ci.parse::<InlineAxis>()) {
            return Ok(PositionFour::BlockInline((ba, b_lp), (ia, d_lp)));
        }
        if let (Ok(ia), Ok(ba)) = (ai.parse::<InlineAxis>(), ci.parse::<BlockAxis>()) {
            return Ok(PositionFour::BlockInline((ba, d_lp), (ia, b_lp)));
        }

        if let (Ok(s1), Ok(s2)) = (ai.parse::<Side>(), ci.parse::<Side>()) {
            return Ok(PositionFour::StartEnd((s1, b_lp), (s2, d_lp)));
        }

        if let (Ok(h), Ok(v)) = (ai.parse::<HorizontalOrXSide>(), ci.parse::<VerticalOrYSide>()) {
            return Ok(PositionFour::XYPercentage((h, b_lp), (v, d_lp)));
        }

        if let (Ok(v), Ok(h)) = (ai.parse::<VerticalOrYSide>(), ci.parse::<HorizontalOrXSide>()) {
            return Ok(PositionFour::XYPercentage((h, d_lp), (v, b_lp)));
        }

        stream.restore(checkpoint);
        Err(format!("Invalid 4-value position: '{}' and '{}' are not a valid axis pair", ai, ci))
    }
}

impl CSSParsable for PositionFour {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        Self::parse_ident_lp_ident_lp(stream)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Position {
    One(PositionOne),
    Two(PositionTwo),
    Four(PositionFour),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BgPosition {
    One(PositionOne),
    Two(PositionTwo),
    Three(PositionThree),
    Four(PositionFour),
}

#[derive(Debug, Clone)]
pub struct BackgroundPosition(pub Vec<BgPosition>);

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

impl CSSParsable for Position {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        let checkpoint = stream.checkpoint();

        if let Ok(v) = PositionFour::parse(stream) {
            if !stream.remaining().is_empty() {
                return Err("Unexpected extra tokens after 4-value position".into());
            }

            return Ok(Position::Four(v));
        }
        stream.restore(checkpoint);

        if PositionThree::parse(stream).is_ok() {
            return Err("3-value positions are not allowed in <position>".into());
        }

        if let Ok(v) = PositionTwo::parse(stream) {
            if !stream.remaining().is_empty() {
                return Err("Unexpected extra tokens after 2-value position".into());
            }

            return Ok(Position::Two(v));
        }
        stream.restore(checkpoint);

        if let Ok(v) = PositionOne::parse(stream) {
            if !stream.remaining().is_empty() {
                return Err("Unexpected extra tokens after 1-value position".into());
            }

            return Ok(Position::One(v));
        }

        Err("Invalid <position>".into())
    }
}

impl CSSParsable for BgPosition {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        let checkpoint = stream.checkpoint();

        if let Ok(v) = PositionFour::parse(stream) {
            if !stream.remaining().is_empty() {
                return Err("Unexpected extra tokens after 4-value position".into());
            }

            return Ok(BgPosition::Four(v));
        }
        stream.restore(checkpoint);

        if let Ok(v) = PositionThree::parse(stream) {
            if !stream.remaining().is_empty() {
                return Err("Unexpected extra tokens after 3-value position".into());
            }

            return Ok(BgPosition::Three(v));
        }
        stream.restore(checkpoint);

        if let Ok(v) = PositionTwo::parse(stream) {
            if !stream.remaining().is_empty() {
                return Err("Unexpected extra tokens after 2-value position".into());
            }

            return Ok(BgPosition::Two(v));
        }
        stream.restore(checkpoint);

        if let Ok(v) = PositionOne::parse(stream) {
            if !stream.remaining().is_empty() {
                return Err("Unexpected extra tokens after 1-value position".into());
            }

            return Ok(BgPosition::One(v));
        }

        Err("Invalid <bg-position>".into())
    }
}

impl CSSParsable for BackgroundPosition {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        let mut layers = Vec::new();
        let mut values = Vec::new();

        while let Some(cv) = stream.next_cv() {
            match cv {
                ComponentValue::Token(t) if t.kind == CssTokenKind::Comma => {
                    if values.is_empty() {
                        return Err("Unexpected comma in background-position".into());
                    }
                    let mut value_stream = ComponentValueStream::from(&values);
                    let pos = BgPosition::parse(&mut value_stream)?;
                    layers.push(pos);
                    values.clear();
                }
                _ => values.push(cv.clone()),
            }
        }

        if !values.is_empty() {
            let mut value_stream = ComponentValueStream::from(&values);
            let pos = BgPosition::parse(&mut value_stream)?;
            layers.push(pos);
        } else if layers.is_empty() {
            return Err("Expected at least one position in background-position".into());
        }

        Ok(BackgroundPosition(layers))
    }
}

#[cfg(test)]
mod tests {
    use css_cssom::{CssToken, NumericValue};

    use super::*;

    #[test]
    fn test_position_one_horizontal() {
        let horizontal = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("left".into()),
            position: None,
        })];

        let x_side = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("x-start".into()),
            position: None,
        })];

        let mut h_stream = ComponentValueStream::from(&horizontal);
        let h_result = PositionOne::parse(&mut h_stream);

        let mut x_stream = ComponentValueStream::from(&x_side);
        let x_result = PositionOne::parse(&mut x_stream);

        assert!(h_result.is_ok());
        assert_eq!(h_result.unwrap(), PositionOne::Horizontal(HorizontalOrXSide::Horizontal(HorizontalSide::Left)));
        assert!(x_result.is_ok());
        assert_eq!(x_result.unwrap(), PositionOne::Horizontal(HorizontalOrXSide::XSide(XSide::XStart)));
    }

    #[test]
    fn test_position_one_center() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("center".into()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = PositionOne::parse(&mut stream);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PositionOne::Center(Center::Center));
    }

    #[test]
    fn test_position_one_vertical() {
        let vertical = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("top".into()),
            position: None,
        })];

        let y_side = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("y-start".into()),
            position: None,
        })];

        let mut v_stream = ComponentValueStream::from(&vertical);
        let v_result = PositionOne::parse(&mut v_stream);

        let mut y_stream = ComponentValueStream::from(&y_side);
        let y_result = PositionOne::parse(&mut y_stream);

        assert!(v_result.is_ok());
        assert_eq!(v_result.unwrap(), PositionOne::Vertical(VerticalOrYSide::Vertical(VerticalSide::Top)));
        assert!(y_result.is_ok());
        assert_eq!(y_result.unwrap(), PositionOne::Vertical(VerticalOrYSide::YSide(YSide::YStart)));
    }

    #[test]
    fn test_position_one_block() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("block-start".into()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = PositionOne::parse(&mut stream);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PositionOne::BlockAxis(BlockAxis::BlockStart));
    }

    #[test]
    fn test_position_one_inline() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("inline-start".into()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = PositionOne::parse(&mut stream);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PositionOne::InlineAxis(InlineAxis::InlineStart));
    }

    #[test]
    fn test_position_one_length() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Dimension {
                value: NumericValue::Number(20.0),
                unit: "px".into(),
            },
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = PositionOne::parse(&mut stream);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PositionOne::LengthPercentage(LengthPercentage::Length(Length::px(20.0))));
    }

    #[test]
    fn test_position_one_percentage() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Percentage(NumericValue::Number(20.0)),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = PositionOne::parse(&mut stream);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PositionOne::LengthPercentage(LengthPercentage::Percentage(Percentage::new(20.0))));
    }

    #[test]
    fn test_position_two_axis() {
        let s_input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("left".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("top".into()),
                position: None,
            }),
        ];

        let i_input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("top".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("left".into()),
                position: None,
            }),
        ];

        let mut s_stream = ComponentValueStream::from(&s_input);
        let s_result = PositionTwo::parse(&mut s_stream);

        let mut i_stream = ComponentValueStream::from(&i_input);
        let i_result = PositionTwo::parse(&mut i_stream);

        assert!(s_result.is_ok());
        let s_value = s_result.unwrap();
        assert_eq!(
            s_value,
            PositionTwo::Axis(XAxis::Horizontal(HorizontalSide::Left), YAxis::Vertical(VerticalSide::Top))
        );
        assert!(i_result.is_ok());
        assert_eq!(i_result.unwrap(), s_value);
    }

    #[test]
    fn test_position_two_axis_and_length_percentage() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Percentage(NumericValue::Number(20.0)),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("top".into()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = PositionTwo::parse(&mut stream);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            PositionTwo::AxisOrPercentage(
                XAxisOrLengthPercentage::LengthPercentage(LengthPercentage::Percentage(Percentage::new(20.0))),
                YAxisOrLengthPercentage::YAxis(YAxis::Vertical(VerticalSide::Top))
            )
        );
    }

    #[test]
    fn test_position_two_block_inline() {
        let s_input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("block-start".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("inline-end".into()),
                position: None,
            }),
        ];

        let i_input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("inline-end".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("block-start".into()),
                position: None,
            }),
        ];

        let mut s_stream = ComponentValueStream::from(&s_input);
        let s_result = PositionTwo::parse(&mut s_stream);

        let mut i_stream = ComponentValueStream::from(&i_input);
        let i_result = PositionTwo::parse(&mut i_stream);

        assert!(s_result.is_ok());
        let s_value = s_result.unwrap();
        assert_eq!(s_value, PositionTwo::BlockInline(BlockAxis::BlockStart, InlineAxis::InlineEnd));
        assert!(i_result.is_ok());
        assert_eq!(i_result.unwrap(), s_value);
    }

    #[test]
    fn test_position_two_relative() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("start".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("center".into()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = PositionTwo::parse(&mut stream);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            PositionTwo::Relative(RelativeAxis::Side(Side::Start), RelativeAxis::Center(Center::Center))
        );
    }

    #[test]
    fn test_position_three_vertical() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("center".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("top".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Dimension {
                    value: NumericValue::Number(20.0),
                    unit: "px".into(),
                },
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = PositionThree::parse(&mut stream);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            PositionThree::RelativeVertical(
                RelativeHorizontalSide::Center(Center::Center),
                (VerticalSide::Top, LengthPercentage::Length(Length::px(20.0)))
            )
        );
    }

    #[test]
    fn test_position_three_horizontal() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("left".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Dimension {
                    value: NumericValue::Number(20.0),
                    unit: "px".into(),
                },
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("center".into()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = PositionThree::parse(&mut stream);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            PositionThree::RelativeHorizontal(
                (HorizontalSide::Left, LengthPercentage::Length(Length::px(20.0))),
                RelativeVerticalSide::Center(Center::Center)
            )
        );
    }

    #[test]
    fn test_position_four_xy() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("left".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Dimension {
                    value: NumericValue::Number(20.0),
                    unit: "px".into(),
                },
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("top".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Dimension {
                    value: NumericValue::Number(20.0),
                    unit: "px".into(),
                },
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = PositionFour::parse(&mut stream);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            PositionFour::XYPercentage(
                (HorizontalOrXSide::Horizontal(HorizontalSide::Left), LengthPercentage::Length(Length::px(20.0))),
                (VerticalOrYSide::Vertical(VerticalSide::Top), LengthPercentage::Length(Length::px(20.0)))
            )
        );
    }

    #[test]
    fn test_position_four_block_inline() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("block-start".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Dimension {
                    value: NumericValue::Number(20.0),
                    unit: "px".into(),
                },
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("inline-start".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Dimension {
                    value: NumericValue::Number(20.0),
                    unit: "px".into(),
                },
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = PositionFour::parse(&mut stream);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            PositionFour::BlockInline(
                (BlockAxis::BlockStart, LengthPercentage::Length(Length::px(20.0))),
                (InlineAxis::InlineStart, LengthPercentage::Length(Length::px(20.0)))
            )
        );
    }

    #[test]
    fn test_position_four_start_end() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("start".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Dimension {
                    value: NumericValue::Number(20.0),
                    unit: "px".into(),
                },
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("end".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Dimension {
                    value: NumericValue::Number(20.0),
                    unit: "px".into(),
                },
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = PositionFour::parse(&mut stream);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            PositionFour::StartEnd(
                (Side::Start, LengthPercentage::Length(Length::px(20.0))),
                (Side::End, LengthPercentage::Length(Length::px(20.0)))
            )
        );
    }

    #[test]
    fn test_position_parse() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("left".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("top".into()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = Position::parse(&mut stream);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Position::Two(PositionTwo::Axis(
                XAxis::Horizontal(HorizontalSide::Left),
                YAxis::Vertical(VerticalSide::Top)
            ))
        );
    }

    #[test]
    fn test_position_fail_three_values() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("center".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("top".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Percentage(NumericValue::Number(20.0)),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = Position::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_position_fail_extra_values() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("center".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("top".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("left".into()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = Position::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_bg_position_multiple_layers() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("left".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("top".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("right".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("bottom".into()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundPosition::parse(&mut stream);

        assert!(result.is_ok());
        let bg_pos = result.unwrap();
        assert_eq!(bg_pos.0.len(), 2);
        assert_eq!(
            bg_pos.0[0],
            BgPosition::Two(PositionTwo::Axis(
                XAxis::Horizontal(HorizontalSide::Left),
                YAxis::Vertical(VerticalSide::Top)
            ))
        );
        assert_eq!(
            bg_pos.0[1],
            BgPosition::Two(PositionTwo::Axis(
                XAxis::Horizontal(HorizontalSide::Right),
                YAxis::Vertical(VerticalSide::Bottom)
            ))
        );
    }

    #[test]
    fn test_bg_position_single_layer() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("center".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("center".into()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundPosition::parse(&mut stream);

        assert!(result.is_ok());
        let bg_pos = result.unwrap();
        assert_eq!(bg_pos.0.len(), 1);
        assert_eq!(
            bg_pos.0[0],
            BgPosition::Two(PositionTwo::Axis(XAxis::Center(Center::Center), YAxis::Center(Center::Center)))
        );
    }
}
