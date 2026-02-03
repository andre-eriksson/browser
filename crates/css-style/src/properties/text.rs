use std::str::FromStr;

use strum::EnumString;

use crate::primitives::{length::Length, percentage::Percentage};

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum Whitespace {
    #[default]
    Normal,
    Pre,
    PreWrap,
    PreLine,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum LineHeight {
    #[default]
    Normal,
    Number(f32),
    Length(Length),
    Percentage(Percentage),
}

impl LineHeight {
    pub fn px(value: f32) -> Self {
        LineHeight::Length(Length::px(value))
    }

    pub fn to_px(self, font_size_px: f32) -> f32 {
        match self {
            LineHeight::Normal => font_size_px * 1.2,
            LineHeight::Number(num) => font_size_px * num,
            LineHeight::Length(len) => len.to_px(0.0, font_size_px),
            LineHeight::Percentage(pct) => pct.to_px(font_size_px),
        }
    }
}

impl FromStr for LineHeight {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("normal") {
            Ok(Self::Normal)
        } else if let Ok(number) = s.parse::<f32>() {
            Ok(Self::Number(number))
        } else if let Ok(length) = s.parse() {
            Ok(Self::Length(length))
        } else if let Ok(percentage) = s.parse() {
            Ok(Self::Percentage(percentage))
        } else {
            Err(format!("Invalid line-height value: {}", s))
        }
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

    #[test]
    fn test_parse_line_height() {
        assert_eq!("normal".parse(), Ok(LineHeight::Normal));
        assert_eq!("1.5".parse(), Ok(LineHeight::Number(1.5)));
        assert_eq!("20px".parse(), Ok(LineHeight::px(20.0)));
        assert_eq!(
            "150%".parse(),
            Ok(LineHeight::Percentage(Percentage::new(150.0)))
        );
        assert!("unknown".parse::<LineHeight>().is_err());
    }
}
