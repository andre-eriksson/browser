use std::str::FromStr;

use crate::primitives::{length::Length, percentage::Percentage};

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub enum Dimension {
    Percentage(Percentage),
    Length(Length),
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
}

impl FromStr for Dimension {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(num) = s.parse::<f32>()
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

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub enum MaxDimension {
    Length(Length),
    Percentage(Percentage),
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
        if s.contains('%')
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
