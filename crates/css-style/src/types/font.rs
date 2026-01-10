use crate::types::{
    global::Global,
    length::{Length, LengthUnit},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

impl GenericName {
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "serif" => Some(GenericName::Serif),
            "sans-serif" => Some(GenericName::SansSerif),
            "monospace" => Some(GenericName::Monospace),
            "cursive" => Some(GenericName::Cursive),
            "fantasy" => Some(GenericName::Fantasy),
            "system-ui" => Some(GenericName::SystemUi),
            "ui-serif" => Some(GenericName::UiSerif),
            "ui-sans-serif" => Some(GenericName::UiSansSerif),
            "ui-monospace" => Some(GenericName::UiMonospace),
            "ui-rounded" => Some(GenericName::UiRounded),
            "math" => Some(GenericName::Math),
            "fangsong" => Some(GenericName::FangSong),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontFamilyName {
    Generic(GenericName),
    Specific(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontFamily {
    pub names: Vec<FontFamilyName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AbsoluteSize {
    XXSmall,
    XSmall,
    Small,
    Medium,
    Large,
    XLarge,
    XXLarge,
    XXXLarge,
}

impl AbsoluteSize {
    pub fn parse(size: &str) -> Option<Self> {
        match size.to_lowercase().as_str() {
            "xx-small" => Some(AbsoluteSize::XXSmall),
            "x-small" => Some(AbsoluteSize::XSmall),
            "small" => Some(AbsoluteSize::Small),
            "medium" => Some(AbsoluteSize::Medium),
            "large" => Some(AbsoluteSize::Large),
            "x-large" => Some(AbsoluteSize::XLarge),
            "xx-large" => Some(AbsoluteSize::XXLarge),
            "xxx-large" => Some(AbsoluteSize::XXXLarge),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RelativeSize {
    Smaller,
    Larger,
}

impl RelativeSize {
    pub fn parse(size: &str) -> Option<Self> {
        match size.to_lowercase().as_str() {
            "smaller" => Some(RelativeSize::Smaller),
            "larger" => Some(RelativeSize::Larger),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FontSize {
    Absolute(AbsoluteSize),
    Relative(RelativeSize),
    Length(Length),
    Percentage(f32),
    Global(Global),
}

impl FontSize {
    pub fn to_px(&self, parent_px: f32) -> f32 {
        match self {
            FontSize::Absolute(abs) => match abs {
                AbsoluteSize::XXSmall => 9.0,
                AbsoluteSize::XSmall => 10.0,
                AbsoluteSize::Small => 13.0,
                AbsoluteSize::Medium => 16.0,
                AbsoluteSize::Large => 18.0,
                AbsoluteSize::XLarge => 24.0,
                AbsoluteSize::XXLarge => 32.0,
                AbsoluteSize::XXXLarge => 48.0,
            },
            FontSize::Length(len) => match len.unit {
                LengthUnit::Px => len.value,
                LengthUnit::Em => len.value * parent_px,
                LengthUnit::Rem => len.value * 16.0,
                LengthUnit::Pt => len.value * 96.0 / 72.0,
                _ => len.value * parent_px,
            },
            FontSize::Percentage(pct) => parent_px * pct / 100.0,
            FontSize::Relative(rel) => match rel {
                RelativeSize::Smaller => parent_px * 0.833,
                RelativeSize::Larger => parent_px * 1.2,
            },
            FontSize::Global(_) => parent_px,
        }
    }
}
