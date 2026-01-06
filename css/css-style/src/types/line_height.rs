use crate::types::{global::Global, length::Length};

#[derive(Debug, Clone)]
pub enum LineHeight {
    Normal,
    Number(f32),
    Length(Length),
    Percentage(f32),
    Global(Global),
}
