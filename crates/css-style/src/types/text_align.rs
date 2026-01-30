use crate::types::{Parseable, global::Global};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TextAlign {
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
    MatchParent,
    Global(Global),
}

impl Parseable for TextAlign {
    fn parse(value: &str) -> Option<Self> {
        if let Some(global) = Global::parse(value) {
            return Some(Self::Global(global));
        }

        if value.eq_ignore_ascii_case("start") {
            return Some(Self::Start);
        } else if value.eq_ignore_ascii_case("end") {
            return Some(Self::End);
        } else if value.eq_ignore_ascii_case("left") {
            return Some(Self::Left);
        } else if value.eq_ignore_ascii_case("right") {
            return Some(Self::Right);
        } else if value.eq_ignore_ascii_case("center") {
            return Some(Self::Center);
        } else if value.eq_ignore_ascii_case("justify") {
            return Some(Self::Justify);
        } else if value.eq_ignore_ascii_case("match-parent") {
            return Some(Self::MatchParent);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_align() {
        assert_eq!(TextAlign::parse("start"), Some(TextAlign::Start));
        assert_eq!(TextAlign::parse("end"), Some(TextAlign::End));
        assert_eq!(TextAlign::parse("left"), Some(TextAlign::Left));
        assert_eq!(TextAlign::parse("right"), Some(TextAlign::Right));
        assert_eq!(TextAlign::parse("center"), Some(TextAlign::Center));
        assert_eq!(TextAlign::parse("justify"), Some(TextAlign::Justify));
        assert_eq!(
            TextAlign::parse("match-parent"),
            Some(TextAlign::MatchParent)
        );
        assert_eq!(
            TextAlign::parse("inherit"),
            Some(TextAlign::Global(Global::Inherit))
        );
        assert_eq!(TextAlign::parse("unknown"), None);
    }
}
