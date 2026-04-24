use css_values::dimension::{MaxSize, Size};

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub enum ComputedSize {
    #[default]
    Auto,
    Fixed,
    Percentage(f64),
    MaxContent,
    MinContent,
    FitContent,
    Stretch,
}

impl From<Size> for ComputedSize {
    fn from(value: Size) -> Self {
        match value {
            Size::Auto => Self::Auto,
            Size::Length(_) | Size::Calc(_) => Self::Fixed,
            Size::Percentage(p) => Self::Percentage(p.as_fraction()),
            Size::MaxContent => Self::MaxContent,
            Size::MinContent => Self::MinContent,
            Size::FitContent => Self::FitContent,
            Size::Stretch => Self::Stretch,
        }
    }
}

impl From<ComputedSize> for Size {
    fn from(value: ComputedSize) -> Self {
        match value {
            ComputedSize::Auto => Self::Auto,
            ComputedSize::Fixed => Self::Auto,
            ComputedSize::Percentage(_) => Self::Auto,
            ComputedSize::MaxContent => Self::MaxContent,
            ComputedSize::MinContent => Self::MinContent,
            ComputedSize::FitContent => Self::FitContent,
            ComputedSize::Stretch => Self::Stretch,
        }
    }
}

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub enum ComputedMaxDimension {
    #[default]
    None,
    Fixed,
    Percentage(f64),
    MaxContent,
    MinContent,
    FitContent,
    Stretch,
}

impl From<MaxSize> for ComputedMaxDimension {
    fn from(value: MaxSize) -> Self {
        match value {
            MaxSize::None => Self::None,
            MaxSize::Length(_) | MaxSize::Calc(_) => Self::Fixed,
            MaxSize::Percentage(p) => Self::Percentage(p.as_fraction()),
            MaxSize::MaxContent => Self::MaxContent,
            MaxSize::MinContent => Self::MinContent,
            MaxSize::FitContent => Self::FitContent,
            MaxSize::Stretch => Self::Stretch,
        }
    }
}

impl From<ComputedMaxDimension> for MaxSize {
    fn from(value: ComputedMaxDimension) -> Self {
        match value {
            ComputedMaxDimension::None => Self::None,
            ComputedMaxDimension::Fixed => Self::None,
            ComputedMaxDimension::Percentage(_) => Self::None,
            ComputedMaxDimension::MaxContent => Self::MaxContent,
            ComputedMaxDimension::MinContent => Self::MinContent,
            ComputedMaxDimension::FitContent => Self::FitContent,
            ComputedMaxDimension::Stretch => Self::Stretch,
        }
    }
}
