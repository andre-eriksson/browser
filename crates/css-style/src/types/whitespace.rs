use crate::types::{Parseable, global::Global};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Whitespace {
    Normal,
    Pre,
    PreWrap,
    PreLine,
    Global(Global),
}

impl Parseable for Whitespace {
    fn parse(value: &str) -> Option<Self> {
        if let Some(global) = Global::parse(value) {
            return Some(Whitespace::Global(global));
        }

        if value.eq_ignore_ascii_case("normal") {
            Some(Whitespace::Normal)
        } else if value.eq_ignore_ascii_case("pre") {
            Some(Whitespace::Pre)
        } else if value.eq_ignore_ascii_case("pre-wrap") {
            Some(Whitespace::PreWrap)
        } else if value.eq_ignore_ascii_case("pre-line") {
            Some(Whitespace::PreLine)
        } else {
            None
        }
    }
}
