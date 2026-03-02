use css_cssom::CssToken;
use strum::EnumString;

use crate::{angle::Angle, percentage::AnglePercentage};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AngleOrZero {
    Angle(Angle),
    Zero,
}

impl TryFrom<&CssToken> for AngleOrZero {
    type Error = String;

    fn try_from(value: &CssToken) -> Result<Self, Self::Error> {
        if let Ok(angle) = Angle::try_from(value) {
            Ok(AngleOrZero::Angle(angle))
        } else {
            Err("Expected an angle, <zero> is unsupported.".to_string())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnglePercentageOrZero {
    AnglePercentage(AnglePercentage),
    Zero,
}

#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum RadialShape {
    Circle,
    Ellipse,
}

#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
pub enum RadialExtent {
    ClosestCorner,
    ClosestSide,
    FarthestCorner,
    FarthestSide,
}
