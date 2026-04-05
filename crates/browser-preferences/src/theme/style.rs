use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Style {
    pub border_radius: f32,
}

impl Default for Style {
    fn default() -> Self {
        Self { border_radius: 5.0 }
    }
}
