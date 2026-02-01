use crate::{
    types::{Parseable, global::Global, length::Length},
    unit::Unit,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineHeight {
    Normal,
    Number(f32),
    Length(Length),
    Percentage(f32),
    Global(Global),
}

impl LineHeight {
    pub fn to_px(&self, font_size_px: f32) -> f32 {
        match self {
            LineHeight::Normal => font_size_px * 1.2,
            LineHeight::Number(num) => font_size_px * num,
            LineHeight::Length(len) => len.to_px(0.0, font_size_px),
            LineHeight::Percentage(pct) => font_size_px * pct / 100.0,
            LineHeight::Global(_) => font_size_px * 1.2, // Placeholder for global values
        }
    }
}

impl Parseable for LineHeight {
    fn parse(value: &str) -> Option<Self> {
        let global = Global::parse(value);
        if let Some(global_value) = global {
            return Some(Self::Global(global_value));
        }

        if value == "normal" {
            return Some(Self::Normal);
        }

        if let Ok(number) = value.parse::<f32>() {
            return Some(Self::Number(number));
        }

        if let Some(length) = Length::parse(value) {
            return Some(Self::Length(length));
        }

        if let Some(percentage) = Unit::resolve_percentage(value) {
            return Some(Self::Percentage(percentage));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::types::length::LengthUnit;

    use super::*;

    #[test]
    fn test_parse_line_height() {
        assert_eq!(LineHeight::parse("normal"), Some(LineHeight::Normal));
        assert_eq!(LineHeight::parse("1.5"), Some(LineHeight::Number(1.5)));
        assert_eq!(
            LineHeight::parse("20px"),
            Some(LineHeight::Length(Length::new(20.0, LengthUnit::Px)))
        );
        assert_eq!(
            LineHeight::parse("150%"),
            Some(LineHeight::Percentage(150.0))
        );
        assert_eq!(
            LineHeight::parse("inherit"),
            Some(LineHeight::Global(Global::Inherit))
        );
        assert_eq!(LineHeight::parse("unknown"), None);
    }
}
