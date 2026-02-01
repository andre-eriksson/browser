use crate::{
    types::{Parseable, global::Global, length::Length},
    unit::Unit,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontFamilyName {
    Generic(GenericName),
    Specific(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontFamily {
    pub names: Vec<FontFamilyName>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    pub fn to_px(&self) -> f32 {
        match self {
            AbsoluteSize::XXSmall => 9.0,
            AbsoluteSize::XSmall => 10.0,
            AbsoluteSize::Small => 13.0,
            AbsoluteSize::Medium => 16.0,
            AbsoluteSize::Large => 18.0,
            AbsoluteSize::XLarge => 24.0,
            AbsoluteSize::XXLarge => 32.0,
            AbsoluteSize::XXXLarge => 48.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RelativeSize {
    Smaller,
    Larger,
}

impl RelativeSize {
    pub fn to_px(&self, parent_px: f32) -> f32 {
        match self {
            RelativeSize::Smaller => parent_px * 0.833,
            RelativeSize::Larger => parent_px * 1.2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
            FontSize::Absolute(abs) => abs.to_px(),
            FontSize::Length(len) => len.to_px(parent_px),
            FontSize::Percentage(pct) => parent_px * pct / 100.0,
            FontSize::Relative(rel) => rel.to_px(parent_px),
            FontSize::Global(_) => parent_px,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

impl Parseable for GenericName {
    fn parse(value: &str) -> Option<Self> {
        match value.len() {
            4 => {
                if value.eq_ignore_ascii_case("math") {
                    Some(Self::Math)
                } else {
                    None
                }
            }
            5 => {
                if value.eq_ignore_ascii_case("serif") {
                    Some(Self::Serif)
                } else {
                    None
                }
            }
            7 => {
                if value.eq_ignore_ascii_case("cursive") {
                    Some(Self::Cursive)
                } else if value.eq_ignore_ascii_case("fantasy") {
                    Some(Self::Fantasy)
                } else {
                    None
                }
            }
            8 => {
                if value.eq_ignore_ascii_case("ui-serif") {
                    Some(Self::UiSerif)
                } else if value.eq_ignore_ascii_case("fangsong") {
                    Some(Self::FangSong)
                } else {
                    None
                }
            }
            9 => {
                if value.eq_ignore_ascii_case("monospace") {
                    Some(Self::Monospace)
                } else if value.eq_ignore_ascii_case("system-ui") {
                    Some(Self::SystemUi)
                } else {
                    None
                }
            }
            10 => {
                if value.eq_ignore_ascii_case("sans-serif") {
                    Some(Self::SansSerif)
                } else if value.eq_ignore_ascii_case("ui-rounded") {
                    Some(Self::UiRounded)
                } else {
                    None
                }
            }
            12 => {
                if value.eq_ignore_ascii_case("ui-monospace") {
                    Some(Self::UiMonospace)
                } else {
                    None
                }
            }
            13 => {
                if value.eq_ignore_ascii_case("ui-sans-serif") {
                    Some(Self::UiSansSerif)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Parseable for FontFamily {
    fn parse(value: &str) -> Option<Self> {
        let families = value
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();

        if families.is_empty() {
            return None;
        }

        Some(Self {
            names: families
                .into_iter()
                .map(|name| {
                    if let Some(generic) = GenericName::parse(&name) {
                        FontFamilyName::Generic(generic)
                    } else {
                        let unquoted = name.trim_matches('"').trim_matches('\'').to_string();
                        FontFamilyName::Specific(unquoted)
                    }
                })
                .collect(),
        })
    }
}

impl Parseable for AbsoluteSize {
    fn parse(value: &str) -> Option<Self> {
        match value.len() {
            5 => {
                if value.eq_ignore_ascii_case("small") {
                    Some(Self::Small)
                } else if value.eq_ignore_ascii_case("large") {
                    Some(Self::Large)
                } else {
                    None
                }
            }
            6 => {
                if value.eq_ignore_ascii_case("medium") {
                    Some(Self::Medium)
                } else {
                    None
                }
            }
            7 => {
                if value.eq_ignore_ascii_case("x-small") {
                    Some(Self::XSmall)
                } else if value.eq_ignore_ascii_case("x-large") {
                    Some(Self::XLarge)
                } else {
                    None
                }
            }
            8 => {
                if value.eq_ignore_ascii_case("xx-small") {
                    Some(Self::XXSmall)
                } else if value.eq_ignore_ascii_case("xx-large") {
                    Some(Self::XXLarge)
                } else {
                    None
                }
            }
            9 => {
                if value.eq_ignore_ascii_case("xxx-large") {
                    Some(Self::XXXLarge)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Parseable for RelativeSize {
    fn parse(value: &str) -> Option<Self> {
        if value.eq_ignore_ascii_case("smaller") {
            Some(RelativeSize::Smaller)
        } else if value.eq_ignore_ascii_case("larger") {
            Some(RelativeSize::Larger)
        } else {
            None
        }
    }
}

impl Parseable for FontSize {
    fn parse(value: &str) -> Option<Self> {
        if let Some(absolute_size) = AbsoluteSize::parse(value) {
            return Some(Self::Absolute(absolute_size));
        }

        if let Some(relative_size) = RelativeSize::parse(value) {
            return Some(Self::Relative(relative_size));
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

impl Parseable for FontWeight {
    fn parse(value: &str) -> Option<Self> {
        if let Ok(weight) = value.parse::<u16>() {
            return match weight {
                100 => Some(Self::Thin),
                200 => Some(Self::ExtraLight),
                300 => Some(Self::Light),
                400 => Some(Self::Normal),
                500 => Some(Self::Medium),
                600 => Some(Self::SemiBold),
                700 => Some(Self::Bold),
                800 => Some(Self::ExtraBold),
                900 => Some(Self::Black),
                _ => None,
            };
        }

        match value.len() {
            4 => {
                if value.eq_ignore_ascii_case("thin") {
                    Some(Self::Thin)
                } else if value.eq_ignore_ascii_case("bold") {
                    Some(Self::Bold)
                } else {
                    None
                }
            }
            5 => {
                if value.eq_ignore_ascii_case("light") {
                    Some(Self::Light)
                } else if value.eq_ignore_ascii_case("black") {
                    Some(Self::Black)
                } else {
                    None
                }
            }
            6 => {
                if value.eq_ignore_ascii_case("normal") {
                    Some(Self::Normal)
                } else if value.eq_ignore_ascii_case("medium") {
                    Some(Self::Medium)
                } else {
                    None
                }
            }
            8 => {
                if value.eq_ignore_ascii_case("semibold") {
                    Some(Self::SemiBold)
                } else {
                    None
                }
            }
            9 => {
                if value.eq_ignore_ascii_case("extrabold") {
                    Some(Self::ExtraBold)
                } else {
                    None
                }
            }
            10 => {
                if value.eq_ignore_ascii_case("extralight") {
                    Some(Self::ExtraLight)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_name_parse() {
        let generic = GenericName::parse("serif").unwrap();
        assert_eq!(generic, GenericName::Serif);

        let generic = GenericName::parse("sans-serif").unwrap();
        assert_eq!(generic, GenericName::SansSerif);

        let generic = GenericName::parse("monospace").unwrap();
        assert_eq!(generic, GenericName::Monospace);
    }

    #[test]
    fn test_font_family_parse() {
        let family = FontFamily::parse("Arial, 'Times New Roman', serif").unwrap();
        assert_eq!(family.names.len(), 3);
        assert_eq!(
            family.names[0],
            FontFamilyName::Specific("Arial".to_string())
        );
        assert_eq!(
            family.names[1],
            FontFamilyName::Specific("Times New Roman".to_string())
        );
        assert_eq!(family.names[2], FontFamilyName::Generic(GenericName::Serif));
    }

    #[test]
    fn test_font_size_parse() {
        let size = FontSize::parse("medium").unwrap();
        assert_eq!(size, FontSize::Absolute(AbsoluteSize::Medium));

        let size = FontSize::parse("larger").unwrap();
        assert_eq!(size, FontSize::Relative(RelativeSize::Larger));

        let size = FontSize::parse("16px").unwrap();
        assert_eq!(size, FontSize::Length(Length::parse("16px").unwrap()));

        let size = FontSize::parse("150%").unwrap();
        assert_eq!(size, FontSize::Percentage(150.0));
    }

    #[test]
    fn test_font_weight_parse() {
        let weight = FontWeight::parse("bold").unwrap();
        assert_eq!(weight, FontWeight::Bold);
        let weight = FontWeight::parse("700").unwrap();
        assert_eq!(weight, FontWeight::Bold);

        let weight = FontWeight::parse("light").unwrap();
        assert_eq!(weight, FontWeight::Light);
        let weight = FontWeight::parse("300").unwrap();
        assert_eq!(weight, FontWeight::Light);

        let weight = FontWeight::parse("invalid");
        assert!(weight.is_none());
    }
}
