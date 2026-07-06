use std::collections::HashMap;

use browser_args::BrowserArgs;
use io::{Entry, Resource, files::PREFERENCES};
use serde::Deserialize;
use storage::Directory;
use tracing::warn;

use crate::theme::Theme;

pub mod theme;

#[derive(Debug, Clone, Deserialize)]
pub struct BrowserPreferences {
    /// A map of available themes, keyed by their name. This field is skipped during serialization and deserialization,
    /// as themes are loaded separately from the preferences file. The themes are loaded from the user data directory and
    /// include default themes (light and dark) as fallbacks.
    #[serde(skip)]
    themes: HashMap<String, Theme>,

    /// The name of the active theme, which should correspond to a key in the `themes` map.
    #[serde(default = "BrowserPreferences::default_theme")]
    theme: String,

    /// Whether to force dark mode on **only** HTML content **NOT** the browser UI, this will not affect the theme of the browser,
    /// This applies a heuristic to text & background colors in HTML and inverts them.
    #[serde(default)]
    force_dark: bool,
}

impl BrowserPreferences {
    /// Maximum allowed file size for the preferences file, set to 10 KiB.
    const MAX_PREFERENCES_FILE_SIZE: Option<usize> = Some(10 * 1024);

    /// Maximum allowed file size for theme files, set to 1 KiB.
    const MAX_THEME_FILE_SIZE: Option<usize> = Some(1024);

    /// Maximum number of theme files to load from the themes directory, set to 100.
    const MAX_THEME_FILES: Option<usize> = Some(100);

    #[must_use]
    pub fn new(dirs: Directory, active_theme: String, force_dark: bool) -> Self {
        Self {
            themes: Self::load_themes(dirs),
            theme: active_theme,
            force_dark,
        }
    }

    pub fn load(args: &BrowserArgs, dirs: Directory) -> Self {
        match Resource::load(PREFERENCES, dirs.clone(), Self::MAX_PREFERENCES_FILE_SIZE) {
            Ok(data) => {
                let Ok(data) = std::str::from_utf8(&data) else {
                    warn!("Failed to parse preferences file as UTF-8, using default settings.");
                    return Self::new(dirs, "light".to_string(), args.preferences.force_dark);
                };

                let Ok(mut config) = toml::from_str::<Self>(data) else {
                    return Self::new(dirs, "light".to_string(), args.preferences.force_dark);
                };

                config.themes = Self::load_themes(dirs);

                if config.theme.is_empty() || !config.themes.contains_key(&config.theme) {
                    warn!(
                        "Active theme \"{}\" not found in {:?}, defaulting to \"light\".",
                        config.theme,
                        config.themes.keys().collect::<Vec<_>>()
                    );
                    config.theme = "light".to_string();
                }

                if args.preferences.force_dark {
                    config.force_dark = true;
                }

                config
            }
            Err(error) => {
                warn!(%error, "Failed to load preferences, using default settings.");

                Self::new(dirs, "light".to_string(), args.preferences.force_dark)
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

    pub fn force_dark(&self) -> bool {
        self.force_dark
    }

    fn load_themes(dirs: Directory) -> HashMap<String, Theme> {
        let mut themes = HashMap::from([
            ("light".to_string(), Theme::light()),
            ("dark".to_string(), Theme::dark()),
        ]);

        let theme_files = match Resource::load_dir(
            Entry::user_data("themes/"),
            dirs,
            Self::MAX_THEME_FILES,
            Self::MAX_THEME_FILE_SIZE,
        ) {
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
