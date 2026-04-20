use std::collections::HashMap;

use io::{Entry, Resource, files::PREFERENCES};
use serde::Deserialize;
use tracing::warn;

use crate::theme::Theme;

pub mod theme;

#[derive(Debug, Clone, Deserialize)]
pub struct BrowserPreferences {
    #[serde(skip)]
    themes: HashMap<String, Theme>,
    #[serde(default = "BrowserPreferences::default_theme")]
    theme: String,
}

impl Default for BrowserPreferences {
    fn default() -> Self {
        Self {
            themes: Self::load_themes(),
            theme: "light".to_string(),
        }
    }
}

impl BrowserPreferences {
    /// Maximum allowed file size for the preferences file, set to 10 KiB.
    const MAX_PREFERENCES_FILE_SIZE: Option<usize> = Some(10 * 1024);

    /// Maximum allowed file size for theme files, set to 1 KiB.
    const MAX_THEME_FILE_SIZE: Option<usize> = Some(1024);

    /// Maximum number of theme files to load from the themes directory, set to 100.
    const MAX_THEME_FILES: Option<usize> = Some(100);

    #[must_use]
    pub fn new(active_theme: String) -> Self {
        Self {
            themes: Self::load_themes(),
            theme: active_theme,
        }
    }

    pub fn load() -> Self {
        match Resource::load(PREFERENCES, Self::MAX_PREFERENCES_FILE_SIZE) {
            Ok(data) => {
                let Ok(data) = std::str::from_utf8(&data) else {
                    warn!("Failed to parse preferences file as UTF-8, using default settings.");
                    return Self::default();
                };

                let Ok(mut config) = toml::from_str::<Self>(data) else {
                    return Self::default();
                };

                config.themes = Self::load_themes();

                if config.theme.is_empty() || !config.themes.contains_key(&config.theme) {
                    warn!(
                        "Active theme \"{}\" not found in {:?}, defaulting to \"light\".",
                        config.theme,
                        config.themes.keys().collect::<Vec<_>>()
                    );
                    config.theme = "light".to_string();
                }

                config
            }
            Err(error) => {
                warn!(%error, "Failed to load preferences, using default settings.");

                Self::default()
            }
        }
    }

    #[must_use]
    pub fn theme_name(&self) -> &str {
        &self.theme
    }

    /// Get the active theme configuration
    ///
    /// # Panics
    /// * If the active theme is not found in the loaded themes, which should never happen due to checks during loading.
    #[must_use]
    pub fn theme(&self) -> &Theme {
        self.themes
            .get(&self.theme)
            .expect("Active theme should always be valid, due to loading checks.")
    }

    fn load_themes() -> HashMap<String, Theme> {
        let mut themes = HashMap::from([
            ("light".to_string(), Theme::light()),
            ("dark".to_string(), Theme::dark()),
        ]);

        let theme_files =
            match Resource::load_dir(Entry::user_data("themes/"), Self::MAX_THEME_FILES, Self::MAX_THEME_FILE_SIZE) {
                Ok(files) => files,
                Err(error) => {
                    warn!(%error, "Failed to load themes from user data directory, using default themes only.");
                    return themes;
                }
            };

        for file in theme_files {
            if let Ok(content) = std::str::from_utf8(&file)
                && let Ok(theme) = toml::from_str::<Theme>(content)
            {
                if theme.name.is_empty() {
                    warn!("Theme with an empty name will be skipped");
                    continue;
                }

                if theme.name.eq_ignore_ascii_case("light") || theme.name.eq_ignore_ascii_case("dark") {
                    warn!("Theme \"{}\" has a reserved name and will be skipped.", theme.name);
                    continue;
                }

                if themes.contains_key(&theme.name) {
                    warn!("Theme \"{}\" already exists and will be overwritten.", theme.name);
                }

                themes.insert(theme.name.clone(), theme);
            }
        }

        themes
    }

    fn default_theme() -> String {
        "light".to_string()
    }
}
