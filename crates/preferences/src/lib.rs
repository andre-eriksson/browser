use constants::files::SETTINGS;
use serde::{Deserialize, Serialize};
use storage::{
    files::read_file_from_config,
    paths::{create_paths, get_config_path},
};

/// Hex color representation as a string.
pub type Color = String;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PresetTheme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub text: Color,
    pub primary: Color,
    pub secondary: Color,
    pub tertiary: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
}

impl Default for Theme {
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

impl From<PresetTheme> for Theme {
    fn from(preset: PresetTheme) -> Self {
        match preset {
            PresetTheme::Light => Theme::default(),
            PresetTheme::Dark => Self {
                background: "#121212".to_string(),
                foreground: "#1E1E1E".to_string(),
                text: "#E0E0E0".to_string(),
                primary: "#BB86FC".to_string(),
                secondary: "#3700B3".to_string(),
                tertiary: "#03DAC6".to_string(),
                success: "#03DAC6".to_string(),
                warning: "#CF6679".to_string(),
                danger: "#CF6679".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BrowserConfig {
    theme: Theme,
}

impl BrowserConfig {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn load() -> Self {
        match read_file_from_config(SETTINGS) {
            Err(_) => {
                let base_path = get_config_path().expect("Failed to get config path");
                let _ = create_paths(&base_path);

                let path = base_path.join(SETTINGS);

                let serialized = toml::to_string(&BrowserConfig::default());

                if serialized.is_err() {
                    eprintln!("Unable to serialize config file");
                    return BrowserConfig::default();
                }

                let res = std::fs::write(path, serialized.unwrap());

                if res.is_err() {
                    eprintln!("Unable to create settings file")
                }

                BrowserConfig::default()
            }
            Ok(data) => {
                let val = str::from_utf8(&data);

                if val.is_err() {
                    return BrowserConfig::default();
                }

                let out = toml::from_str(val.unwrap());

                if out.is_err() {
                    return BrowserConfig::default();
                }

                let config: BrowserConfig = out.unwrap();

                config
            }
        }
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        if let Some(base_path) = get_config_path() {
            let path = base_path.join(SETTINGS);
            let serialized = toml::to_string(self);
            if let Ok(serialized) = serialized {
                let _ = std::fs::write(path, serialized);
            }
        }
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }
}
