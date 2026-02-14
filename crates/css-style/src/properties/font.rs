use std::str::FromStr;

use css_cssom::{ComponentValue, CssTokenKind};

use crate::{
    ComputedStyle, RelativeType,
    calculate::CalcExpression,
    length::LengthUnit,
    primitives::{
        font::{AbsoluteSize, GenericName, RelativeSize},
        length::Length,
        percentage::Percentage,
    },
    properties::{AbsoluteContext, RelativeContext},
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

impl TryFrom<&[ComponentValue]> for FontWeight {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("normal") {
                            return Ok(FontWeight::Normal);
                        } else if ident.eq_ignore_ascii_case("bold") {
                            return Ok(FontWeight::Bold);
                        }
                    }
                    CssTokenKind::Number(num) => {
                        if let Ok(weight) = FontWeight::try_from(num.value as u16) {
                            return Ok(weight);
                        }
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Err(format!("Invalid font weight value: {:?}", value))
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

impl TryFrom<&[ComponentValue]> for FontFamily {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let mut full: Vec<FontFamilyName> = Vec::with_capacity(4);
        let mut names: Vec<String> = Vec::with_capacity(4);

        for cv in value {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        names.push(ident.clone());
                    }
                    CssTokenKind::String(s) => {
                        names.push(s.clone());
                    }
                    CssTokenKind::Comma => {
                        let full_name = names.join(" ");
                        if let Ok(generic) = full_name.parse() {
                            full.push(FontFamilyName::Generic(generic));
                        } else {
                            full.push(FontFamilyName::Specific(full_name));
                        }
                        names.clear();
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        if names.is_empty() {
            if full.is_empty() {
                return Err("No valid font family names found".to_string());
            }
        } else {
            let full_name = names.join(" ");
            if let Ok(generic) = full_name.parse() {
                full.push(FontFamilyName::Generic(generic));
            } else {
                full.push(FontFamilyName::Specific(full_name));
            }
        }

        Ok(FontFamily { names: full })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontSize {
    Absolute(AbsoluteSize),
    Relative(RelativeSize),
    Length(Length),
    Percentage(Percentage),
    Calc(CalcExpression),
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

    pub fn to_px(&self, abs_ctx: &AbsoluteContext, font_size_px: f32) -> f32 {
        let rel_ctx = RelativeContext {
            parent: ComputedStyle {
                font_size: font_size_px,
                ..Default::default()
            }
            .into(),
        };

        match self {
            FontSize::Absolute(abs) => abs.to_px(),
            FontSize::Length(len) => len.to_px(&rel_ctx, abs_ctx),
            FontSize::Percentage(pct) => pct.as_fraction() * rel_ctx.parent.font_size,
            FontSize::Relative(rel) => rel.to_px(rel_ctx.parent.font_size),
            FontSize::Calc(calc) => calc.to_px(Some(RelativeType::FontSize), &rel_ctx, abs_ctx),
        }
    }
}

impl TryFrom<&[ComponentValue]> for FontSize {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Function(func) => {
                    if func.name.eq_ignore_ascii_case("calc") {
                        return Ok(FontSize::Calc(CalcExpression::parse(
                            func.value.as_slice(),
                        )?));
                    }
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if let Ok(abs_size) = ident.parse() {
                            return Ok(FontSize::Absolute(abs_size));
                        } else if let Ok(rel_size) = ident.parse() {
                            return Ok(FontSize::Relative(rel_size));
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| format!("Invalid length unit: {}", unit))?;
                        return Ok(FontSize::Length(Length::new(value.value as f32, len_unit)));
                    }
                    CssTokenKind::Percentage(num) => {
                        return Ok(FontSize::Percentage(Percentage::new(num.value as f32)));
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Err(format!("Invalid font size value: {:?}", value))
    }
}

#[cfg(test)]
mod tests {
    use css_cssom::CssToken;

    use crate::primitives::font::GenericName;

    use super::*;

    #[test]
    fn test_font_family_ident_parse() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("Times".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("New".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("Roman".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("serif".to_string()),
                position: None,
            }),
        ];

        let font_family = FontFamily::try_from(input.as_slice()).unwrap();
        assert_eq!(font_family.names().len(), 2);
        assert_eq!(
            font_family.names()[0],
            FontFamilyName::Specific("Times New Roman".to_string())
        );
        assert_eq!(
            font_family.names()[1],
            FontFamilyName::Generic(GenericName::Serif)
        );
    }

    #[test]
    fn test_font_family_string_parse() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::String("Open Sans".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("serif".to_string()),
                position: None,
            }),
        ];

        let font_family = FontFamily::try_from(input.as_slice()).unwrap();
        assert_eq!(font_family.names().len(), 2);
        assert_eq!(
            font_family.names()[0],
            FontFamilyName::Specific("Open Sans".to_string())
        );
        assert_eq!(
            font_family.names()[1],
            FontFamilyName::Generic(GenericName::Serif)
        );
    }
}
