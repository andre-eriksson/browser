use std::collections::HashMap;

use io::{Resource, files::PREFERENCES};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeCategory {
    #[default]
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub category: ThemeCategory,
    pub background: String,
    pub foreground: String,
    pub text: String,
    pub primary: String,
    pub secondary: String,
    pub tertiary: String,
    pub success: String,
    pub warning: String,
    pub danger: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            category: ThemeCategory::Light,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeCollection {
    #[serde(default)]
    light: Theme,
    #[serde(flatten)]
    extras: HashMap<String, Theme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    theme: ThemeCollection,
    active_theme: String,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            theme: ThemeCollection::default(),
            active_theme: "light".to_string(),
        }
    }
}

impl BrowserConfig {
    pub fn new(active_theme: String) -> Self {
        Self {
            theme: ThemeCollection::default(),
            active_theme,
        }
    }

    pub fn load() -> Self {
        match Resource::load(PREFERENCES) {
            Err(_) => {
                let serialized = toml::to_string(&Self::default());

                if serialized.is_err() {
                    warn!("Unable to serialize config file");
                    return Self::default();
                }

                let res = Resource::write(PREFERENCES, serialized.unwrap());

                if res.is_err() {
                    warn!("Unable to create settings file")
                }

                Self::default()
            }
            Ok(data) => {
                let val = str::from_utf8(&data);

                if val.is_err() {
                    return Self::default();
                }

                let out = toml::from_str(val.unwrap());

                if out.is_err() {
                    return Self::default();
                }

                let config: BrowserConfig = out.unwrap();

                if config.active_theme.is_empty() {
                    warn!("Active theme is empty, defaulting to 'light'");
                    return Self::default();
                }

                config
            }
        }
    }

    pub fn set_active_theme(&mut self, theme: String) {
        self.active_theme = theme;
    }

    pub fn active_theme(&self) -> &Theme {
        self.theme
            .extras
            .get(&self.active_theme)
            .unwrap_or(&self.theme.light)
    }
}
