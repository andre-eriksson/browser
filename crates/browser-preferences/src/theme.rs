use serde::Deserialize;

use crate::theme::{
    colors::Color,
    style::Style,
    typography::{FontDescriptor, Typography},
};

mod colors;
mod style;
mod typography;

/// Represents the category of a theme, either Light or Dark, this will impact the CSS functions that need the colors to be
/// adjusted for light or dark themes, e.g. using `light-dark()` to adjust the colors for light and dark themes.
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeCategory {
    #[default]
    Light,
    Dark,
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

    /// The style settings for the theme, e.g. border radius, etc.
    pub style: Style,
}

impl Theme {
    /// Creates a new light theme with default values.
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
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

            style: Style { border_radius: 5.0 },
        }
    }

    /// Creates a new dark theme with default values.
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
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

            style: Style { border_radius: 5.0 },
        }
    }
}
