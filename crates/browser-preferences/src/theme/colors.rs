use serde::Deserialize;

/// Represents the color palette for a theme, including background, foreground, text, and accent colors.
///
/// They are all specified in hex format, e.g. "#FFFFFF" for white, "#000000" for black, etc.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Color {
    /// The background color which is behind/underneath the HTML content.
    pub background: String,

    /// The foreground color which is the background of the UI elements, e.g. the tab bar, the address bar, the bookmarks bar, etc.
    pub foreground: String,

    /// The text color which is the color of the text in the UI elements, e.g. the tab bar, the address bar, the bookmarks bar, etc.
    ///
    /// This does not affect the text color of any HTML content.
    pub text: String,

    /// The primary color which is used for the primary actions, e.g. the active tab, etc.
    pub primary: String,

    /// The secondary color which is used for the secondary actions, e.g. the inactive tabs, etc.
    pub secondary: String,

    /// The tertiary color which is used for the tertiary actions, e.g. the new tab button, etc.
    pub tertiary: String,

    /// The success color which is used for the success actions, e.g. the success messages, etc.
    pub success: String,

    /// The warning color which is used for the warning actions, e.g. the warning messages, etc.
    pub warning: String,

    /// The danger color which is used for the danger actions, e.g. the error messages, etc.
    pub danger: String,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            background: "#FFFFFF".to_string(),
            foreground: "#F6F8FB".to_string(),
            text: "#1A1A1A".to_string(),
            primary: "#5BC0EB".to_string(),
            secondary: "#9BD7F5".to_string(),
            tertiary: "#3A86FF".to_string(),
            success: "#8AC926".to_string(),
            warning: "#FFB703".to_string(),
            danger: "#EF476F".to_string(),
        }
    }
}
