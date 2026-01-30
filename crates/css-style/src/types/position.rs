//! <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/position>
use crate::types::{Parseable, global::Global};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
    Global(Global),
}

impl Parseable for Position {
    fn parse(value: &str) -> Option<Self> {
        if let Some(global) = Global::parse(value) {
            return Some(Self::Global(global));
        }

        if value.eq_ignore_ascii_case("static") {
            return Some(Self::Static);
        } else if value.eq_ignore_ascii_case("relative") {
            return Some(Self::Relative);
        } else if value.eq_ignore_ascii_case("absolute") {
            return Some(Self::Absolute);
        } else if value.eq_ignore_ascii_case("fixed") {
            return Some(Self::Fixed);
        } else if value.eq_ignore_ascii_case("sticky") {
            return Some(Self::Sticky);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_position() {
        assert_eq!(Position::parse("static"), Some(Position::Static));
        assert_eq!(Position::parse("relative"), Some(Position::Relative));
        assert_eq!(Position::parse("absolute"), Some(Position::Absolute));
        assert_eq!(Position::parse("fixed"), Some(Position::Fixed));
        assert_eq!(Position::parse("sticky"), Some(Position::Sticky));
        assert_eq!(
            Position::parse("inherit"),
            Some(Position::Global(Global::Inherit))
        );
        assert_eq!(Position::parse("unknown"), None);
    }
}
