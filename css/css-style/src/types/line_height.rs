use crate::types::{global::Global, length::Length};

#[derive(Debug, Clone)]
pub enum LineHeight {
    Normal,
    Number(f32),
    Length(Length),
    Percentage(f32),
    Global(Global),
}

impl LineHeight {
    pub fn to_px(&self, font_size_px: f32) -> f32 {
        match self {
            LineHeight::Normal => font_size_px * 1.2,
            LineHeight::Number(num) => font_size_px * num,
            LineHeight::Length(len) => len.to_px(font_size_px),
            LineHeight::Percentage(pct) => font_size_px * pct / 100.0,
            LineHeight::Global(_) => font_size_px * 1.2, // Placeholder for global values
        }
    }
}
