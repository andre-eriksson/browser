use crate::{Dimension, MaxDimension, length::Length};

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub enum ComputedDimension {
    #[default]
    Auto,
    MaxContent,
    MinContent,
    FitContent(Option<Length>),
    Stretch,
}

impl TryFrom<Dimension> for ComputedDimension {
    type Error = String;

    fn try_from(value: Dimension) -> Result<Self, Self::Error> {
        match value {
            Dimension::Auto => Ok(Self::Auto),
            Dimension::MaxContent => Ok(Self::MaxContent),
            Dimension::MinContent => Ok(Self::MinContent),
            Dimension::FitContent(len) => Ok(Self::FitContent(len)),
            Dimension::Stretch => Ok(Self::Stretch),
            _ => Err(format!("Unsupported Dimension value: {:?}", value)),
        }
    }
}

impl From<ComputedDimension> for Dimension {
    fn from(value: ComputedDimension) -> Self {
        match value {
            ComputedDimension::Auto => Self::Auto,
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
    MaxContent,
    MinContent,
    FitContent(Option<Length>),
    Stretch,
}

impl TryFrom<MaxDimension> for ComputedMaxDimension {
    type Error = String;

    fn try_from(value: MaxDimension) -> Result<Self, Self::Error> {
        match value {
            MaxDimension::None => Ok(Self::None),
            MaxDimension::MaxContent => Ok(Self::MaxContent),
            MaxDimension::MinContent => Ok(Self::MinContent),
            MaxDimension::FitContent(len) => Ok(Self::FitContent(len)),
            MaxDimension::Stretch => Ok(Self::Stretch),
            _ => Err(format!("Unsupported MaxDimension value: {:?}", value)),
        }
    }
}

impl From<ComputedMaxDimension> for MaxDimension {
    fn from(value: ComputedMaxDimension) -> Self {
        match value {
            ComputedMaxDimension::None => Self::None,
            ComputedMaxDimension::MaxContent => Self::MaxContent,
            ComputedMaxDimension::MinContent => Self::MinContent,
            ComputedMaxDimension::FitContent(len) => Self::FitContent(len),
            ComputedMaxDimension::Stretch => Self::Stretch,
        }
    }
}
