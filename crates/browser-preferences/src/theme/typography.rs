use serde::Deserialize;

/// Represents the font description.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FontDescriptor {
    pub name: String,
    pub size: f32,
}

impl Default for FontDescriptor {
    fn default() -> Self {
        Self {
            name: "Open Sans".to_string(),
            size: 16.0,
        }
    }
}

/// Represents the typography settings for the browser.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Typography {
    /// The font descriptor for UI elements, including font family and size.
    pub ui: FontDescriptor,
}
