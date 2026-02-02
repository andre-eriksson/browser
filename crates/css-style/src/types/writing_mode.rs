use crate::types::{Parseable, global::Global};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WritingMode {
    HorizontalTb,
    VerticalRl,
    VerticalLr,
    SidewaysRl,
    SidewaysLr,
    Global(Global),
}

impl Parseable for WritingMode {
    fn parse(value: &str) -> Option<Self> {
        if let Some(global) = Global::parse(value) {
            return Some(WritingMode::Global(global));
        }

        if value.eq_ignore_ascii_case("horizontal-tb") {
            Some(WritingMode::HorizontalTb)
        } else if value.eq_ignore_ascii_case("vertical-rl") {
            Some(WritingMode::VerticalRl)
        } else if value.eq_ignore_ascii_case("vertical-lr") {
            Some(WritingMode::VerticalLr)
        } else if value.eq_ignore_ascii_case("sideways-rl") {
            Some(WritingMode::SidewaysRl)
        } else if value.eq_ignore_ascii_case("sideways-lr") {
            Some(WritingMode::SidewaysLr)
        } else {
            None
        }
    }
}
