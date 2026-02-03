use strum::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
pub enum GenericName {
    Serif,
    SansSerif,
    Monospace,
    Cursive,
    Fantasy,
    SystemUi,
    UiSerif,
    UiSansSerif,
    UiMonospace,
    UiRounded,
    Math,
    FangSong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
pub enum AbsoluteSize {
    XxSmall,
    XSmall,
    Small,
    Medium,
    Large,
    XLarge,
    XxLarge,
    XxxLarge,
}

impl AbsoluteSize {
    pub fn to_px(self) -> f32 {
        match self {
            AbsoluteSize::XxSmall => 9.0,
            AbsoluteSize::XSmall => 10.0,
            AbsoluteSize::Small => 13.0,
            AbsoluteSize::Medium => 16.0,
            AbsoluteSize::Large => 18.0,
            AbsoluteSize::XLarge => 24.0,
            AbsoluteSize::XxLarge => 32.0,
            AbsoluteSize::XxxLarge => 48.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum RelativeSize {
    Smaller,
    Larger,
}

impl RelativeSize {
    pub fn to_px(self, parent_px: f32) -> f32 {
        match self {
            RelativeSize::Smaller => parent_px * 0.833,
            RelativeSize::Larger => parent_px * 1.2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_name_parse() {
        assert_eq!("serif".parse(), Ok(GenericName::Serif));
        assert_eq!("sans-serif".parse(), Ok(GenericName::SansSerif));
        assert_eq!("monospace".parse(), Ok(GenericName::Monospace));
    }
}
