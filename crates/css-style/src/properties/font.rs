use std::str::FromStr;

use crate::primitives::{
    font::{AbsoluteSize, GenericName, RelativeSize},
    length::Length,
    percentage::Percentage,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    #[default]
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

impl TryFrom<u16> for FontWeight {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            100 => Ok(FontWeight::Thin),
            200 => Ok(FontWeight::ExtraLight),
            300 => Ok(FontWeight::Light),
            400 => Ok(FontWeight::Normal),
            500 => Ok(FontWeight::Medium),
            600 => Ok(FontWeight::SemiBold),
            700 => Ok(FontWeight::Bold),
            800 => Ok(FontWeight::ExtraBold),
            900 => Ok(FontWeight::Black),
            _ => Err(format!("Invalid font weight numeric value: {}", value)),
        }
    }
}

impl FromStr for FontWeight {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(num) = s.parse::<u16>()
            && let Ok(font) = FontWeight::try_from(num)
        {
            Ok(font)
        } else if s.eq_ignore_ascii_case("normal") {
            Ok(FontWeight::Normal)
        } else if s.eq_ignore_ascii_case("bold") {
            Ok(FontWeight::Bold)
        } else {
            Err(format!("Invalid font weight value: {}", s))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontFamilyName {
    Generic(GenericName),
    Specific(String),
}

impl Default for FontFamilyName {
    fn default() -> Self {
        FontFamilyName::Generic(GenericName::Serif)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontFamily {
    names: Vec<FontFamilyName>,
}

impl Default for FontFamily {
    fn default() -> Self {
        FontFamily {
            names: vec![FontFamilyName::default()],
        }
    }
}

impl FontFamily {
    pub fn new(names: &[FontFamilyName]) -> Self {
        Self {
            names: names.to_vec(),
        }
    }

    pub fn names(&self) -> &Vec<FontFamilyName> {
        &self.names
    }
}

impl FromStr for FontFamily {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let names = s
            .split(',')
            .map(|name| name.trim())
            .map(|name| {
                if let Ok(generic) = name.parse() {
                    FontFamilyName::Generic(generic)
                } else {
                    let unquoted = name.trim_matches('\'').trim_matches('"').to_string();
                    FontFamilyName::Specific(unquoted)
                }
            })
            .collect();

        Ok(FontFamily { names })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontSize {
    Absolute(AbsoluteSize),
    Relative(RelativeSize),
    Length(Length),
    Percentage(Percentage),
}

impl Default for FontSize {
    fn default() -> Self {
        FontSize::Absolute(AbsoluteSize::Medium)
    }
}

impl FontSize {
    pub fn px(value: f32) -> Self {
        Self::Length(Length::px(value))
    }

    pub fn to_px(self, parent_px: f32) -> f32 {
        match self {
            FontSize::Absolute(abs) => abs.to_px(),
            FontSize::Length(len) => len.to_px(0.0, parent_px),
            FontSize::Percentage(pct) => pct.to_px(parent_px),
            FontSize::Relative(rel) => rel.to_px(parent_px),
        }
    }
}

impl FromStr for FontSize {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if let Ok(abs_size) = s.parse() {
            Ok(FontSize::Absolute(abs_size))
        } else if let Ok(rel_size) = s.parse() {
            Ok(FontSize::Relative(rel_size))
        } else if s.ends_with('%') {
            Ok(FontSize::Percentage(s.parse()?))
        } else if let Ok(length) = s.parse() {
            Ok(FontSize::Length(length))
        } else {
            Err(format!("Invalid font size value: {}", s))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::primitives::font::GenericName;

    use super::*;

    #[test]
    fn test_font_family_parse() {
        let family = "Arial, 'Times New Roman', serif"
            .parse::<FontFamily>()
            .unwrap();
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
        assert_eq!(
            "medium".parse(),
            Ok(FontSize::Absolute(AbsoluteSize::Medium))
        );

        assert_eq!(
            "larger".parse(),
            Ok(FontSize::Relative(RelativeSize::Larger))
        );
        assert_eq!("16px".parse(), Ok(FontSize::px(16.0)));
        assert_eq!(
            "150%".parse(),
            Ok(FontSize::Percentage(Percentage::new(150.0)))
        );
    }
}
