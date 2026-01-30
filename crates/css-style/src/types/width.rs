use crate::types::{global::Global, length::Length};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Width {
    Percentage(f32),
    Length(Length),
    Auto,
    MaxContent,
    MinContent,
    FitContent(Option<Length>),
    Stretch,
    Global(Global),
}
