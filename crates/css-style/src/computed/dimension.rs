use crate::{Dimension, MaxDimension, length::Length};

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub enum ComputedDimension {
    #[default]
    Auto,
    Fixed,
    Percentage(f32),
    MaxContent,
    MinContent,
    FitContent(Option<Length>),
    Stretch,
}

impl From<Dimension> for ComputedDimension {
    fn from(value: Dimension) -> Self {
        match value {
            Dimension::Auto => Self::Auto,
            Dimension::Length(_) | Dimension::Calc(_) => Self::Fixed,
            Dimension::Percentage(p) => Self::Percentage(p.as_fraction()),
            Dimension::MaxContent => Self::MaxContent,
            Dimension::MinContent => Self::MinContent,
            Dimension::FitContent(len) => Self::FitContent(len),
            Dimension::Stretch => Self::Stretch,
        }
    }
}

impl From<ComputedDimension> for Dimension {
    fn from(value: ComputedDimension) -> Self {
        match value {
            ComputedDimension::Auto => Self::Auto,
            ComputedDimension::Fixed => Self::Auto,
            ComputedDimension::Percentage(_) => Self::Auto,
            ComputedDimension::MaxContent => Self::MaxContent,
            ComputedDimension::MinContent => Self::MinContent,
            ComputedDimension::FitContent(len) => Self::FitContent(len),
            ComputedDimension::Stretch => Self::Stretch,
        }
    }
}

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub enum ComputedMaxDimension {
    #[default]
    None,
    Fixed,
    MaxContent,
    MinContent,
    FitContent(Option<Length>),
    Stretch,
}

impl From<MaxDimension> for ComputedMaxDimension {
    fn from(value: MaxDimension) -> Self {
        match value {
            MaxDimension::None => Self::None,
            MaxDimension::Length(_) | MaxDimension::Percentage(_) | MaxDimension::Calc(_) => Self::Fixed,
            MaxDimension::MaxContent => Self::MaxContent,
            MaxDimension::MinContent => Self::MinContent,
            MaxDimension::FitContent(len) => Self::FitContent(len),
            MaxDimension::Stretch => Self::Stretch,
        }
    }
}

impl From<ComputedMaxDimension> for MaxDimension {
    fn from(value: ComputedMaxDimension) -> Self {
        match value {
            ComputedMaxDimension::None => Self::None,
            ComputedMaxDimension::Fixed => Self::None,
            ComputedMaxDimension::MaxContent => Self::MaxContent,
            ComputedMaxDimension::MinContent => Self::MinContent,
            ComputedMaxDimension::FitContent(len) => Self::FitContent(len),
            ComputedMaxDimension::Stretch => Self::Stretch,
        }
    }
}
