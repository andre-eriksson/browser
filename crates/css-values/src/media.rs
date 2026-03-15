use strum::EnumString;

use crate::quantity::Resolution;

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum MediaType {
    All,
    Print,
    Screen,

    // Deprecated media types
    Tty,
    Tv,
    Projection,
    Handheld,
    Braille,
    Embossed,
    Aural,
    Speech,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
pub enum MediaFeature {
    PrefersColorScheme,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
    Only,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum Hover {
    None,
    Hover,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum Pointer {
    None,
    Coarse,
    Fine,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum ColorGamut {
    Srgb,
    P3,
    Rec2020,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum OverflowBlock {
    None,
    Scroll,
    Paged,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum OverflowInline {
    None,
    Scroll,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MediaResolution {
    Resolution(Resolution),
    Infinite,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum Scan {
    Interlace,
    Progressive,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum Update {
    Slow,
    Fast,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RangeOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
}

/// A media condition represents a single media feature and its value, such as (min-width: 600px)
///
/// <https://drafts.csswg.org/mediaqueries/#media-descriptor-table>
#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
pub enum MediaCondition {
    AnyHover,
    AnyPointer,
    AspectRatio,
    Color,
    ColorGamut,
    ColorIndex,
    DeviceAspectRatio,
    DeviceHeight,
    DeviceWidth,
    Grid,
    Height,
    Hover,
    Monochrome,
    Orientation,
    OverflowBlock,
    OverflowInline,
    Pointer,
    Resolution,
    Scan,
    Update,
    Width,
}

impl MediaCondition {
    pub fn is_discrete_query(&self) -> bool {
        matches!(
            self,
            MediaCondition::AnyHover
                | MediaCondition::AnyPointer
                | MediaCondition::ColorGamut
                | MediaCondition::Grid
                | MediaCondition::Hover
                | MediaCondition::Orientation
                | MediaCondition::OverflowBlock
                | MediaCondition::OverflowInline
                | MediaCondition::Pointer
                | MediaCondition::Scan
                | MediaCondition::Update
        )
    }

    pub fn is_range_query(&self) -> bool {
        matches!(
            self,
            MediaCondition::AspectRatio
                | MediaCondition::Color
                | MediaCondition::ColorIndex
                | MediaCondition::DeviceAspectRatio
                | MediaCondition::DeviceHeight
                | MediaCondition::DeviceWidth
                | MediaCondition::Height
                | MediaCondition::Monochrome
                | MediaCondition::Resolution
        )
    }
}
