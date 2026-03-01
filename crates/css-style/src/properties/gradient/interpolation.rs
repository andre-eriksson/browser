use std::str::FromStr;

use strum::EnumString;

#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
pub enum XyzSpace {
    Xyz,
    XyzD50,
    XyzD65,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RectangularColorSpace {
    Srgb,
    SrgbLinear,
    DisplayP3,
    DisplayP3Linear,
    A98Rgb,
    ProphotoRgb,
    Rec2020,
    Lab,
    Oklab,
    Xyz(XyzSpace),
}

impl FromStr for RectangularColorSpace {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(xyz) = s.parse::<XyzSpace>() {
            Ok(Self::Xyz(xyz))
        } else if s.eq_ignore_ascii_case("srgb") {
            Ok(Self::Srgb)
        } else if s.eq_ignore_ascii_case("srgb-linear") {
            Ok(Self::SrgbLinear)
        } else if s.eq_ignore_ascii_case("display-p3") {
            Ok(Self::DisplayP3)
        } else if s.eq_ignore_ascii_case("display-p3-linear") {
            Ok(Self::DisplayP3Linear)
        } else if s.eq_ignore_ascii_case("a98-rgb") {
            Ok(Self::A98Rgb)
        } else if s.eq_ignore_ascii_case("prophoto-rgb") {
            Ok(Self::ProphotoRgb)
        } else if s.eq_ignore_ascii_case("rec2020") {
            Ok(Self::Rec2020)
        } else if s.eq_ignore_ascii_case("lab") {
            Ok(Self::Lab)
        } else if s.eq_ignore_ascii_case("oklab") {
            Ok(Self::Oklab)
        } else {
            Err(strum::ParseError::VariantNotFound)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum PolarColorSpace {
    Hsl,
    Hwb,
    Lch,
    Oklch,
}

#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum HueInterpolationMethod {
    Shorter,
    Longer,
    Increasing,
    Decreasing,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorInterpolationMethod {
    Rectangular(RectangularColorSpace),
    Polar(PolarColorSpace, Option<HueInterpolationMethod>),
    Custom(String),
}
