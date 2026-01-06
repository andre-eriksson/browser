use crate::types::{global::Global, length::Length};

#[derive(Debug, Default, Clone)]
pub enum Width {
    Percentage(f32),
    Length(Length),
    #[default]
    Auto,
    MaxContent,
    MinContent,
    FitContent(Option<Length>),
    Stretch,
    Global(Global),
}
