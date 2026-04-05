use serde::Deserialize;

/// Represents the category of a theme, either Light or Dark, this will impact the CSS functions that need the colors to be
/// adjusted for light or dark themes, e.g. using `light-dark()` to adjust the colors for light and dark themes.
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeCategory {
    #[default]
    Light,
    Dark,
}

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

/// Represents a theme for the browser, including colors and typography settings.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Theme {
    /// The name of the theme, e.g. "Light Theme" or "Dark Theme".
    pub name: String,

    /// The category of the theme, either Light or Dark.
    pub category: ThemeCategory,

    /// The color palette for the theme, including background, foreground, text, and accent colors.
    pub colors: Color,

    /// The typography settings for the theme.
    pub typography: Typography,
}

impl Theme {
    /// Creates a new light theme with default values.
    pub fn light() -> Self {
        Self {
            name: "Light Theme".to_string(),
            category: ThemeCategory::Light,

            colors: Color {
                background: "#FFFFFF".to_string(),
                foreground: "#F6F8FB".to_string(),
                text: "#1A1A1A".to_string(),
                primary: "#5BC0EB".to_string(),
                secondary: "#9BD7F5".to_string(),
                tertiary: "#3A86FF".to_string(),
                success: "#8AC926".to_string(),
                warning: "#FFB703".to_string(),
                danger: "#EF476F".to_string(),
            },

            typography: Typography {
                ui: FontDescriptor {
                    name: "Open Sans".to_string(),
                    size: 16.0,
                },
            },
        }
    }

    /// Creates a new dark theme with default values.
    pub fn dark() -> Self {
        Self {
            name: "Dark Theme".to_string(),
            category: ThemeCategory::Dark,

            colors: Color {
                background: "#1A1A1A".to_string(),
                foreground: "#343434".to_string(),
                text: "#FFFFFF".to_string(),
                primary: "#5BC0EB".to_string(),
                secondary: "#9BD7F5".to_string(),
                tertiary: "#3A86FF".to_string(),
                success: "#8AC926".to_string(),
                warning: "#FFB703".to_string(),
                danger: "#EF476F".to_string(),
            },

            typography: Typography {
                ui: FontDescriptor {
                    name: "Open Sans".to_string(),
                    size: 16.0,
                },
            },
        }
    }
}
