use constants::files::SETTINGS;
use serde::{Deserialize, Serialize};
use storage::{
    files::read_file_from_config,
    paths::{create_paths, get_config_path},
};

/// Hex color representation as a string.
pub type Color = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub text: Color,
    pub primary: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: "#FFFFFF".to_string(),
            foreground: "#D4D4D4".to_string(),
            text: "#0A0A0A".to_string(),
            primary: "#00BBF9".to_string(),
            success: "#90BE6D".to_string(),
            warning: "#F8961E".to_string(),
            danger: "#F94144".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    theme: Theme,
}

impl Config {
    pub fn load() -> Self {
        match read_file_from_config(SETTINGS) {
            Err(_) => {
                let base_path = get_config_path().expect("Failed to get config path");
                let _ = create_paths(&base_path);

                let path = base_path.join(SETTINGS);

                let serialized = toml::to_string(&Config::default());

                if serialized.is_err() {
                    eprintln!("Unable to serialize config file");
                    return Config::default();
                }

                let res = std::fs::write(path, serialized.unwrap());

                if res.is_err() {
                    eprintln!("Unable to create settings file")
                }

                Config::default()
            }
            Ok(data) => {
                let val = str::from_utf8(&data);

                if val.is_err() {
                    return Config::default();
                }

                let out = toml::from_str(val.unwrap());

                if out.is_err() {
                    return Config::default();
                }

                let config: Config = out.unwrap();

                config
            }
        }
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }
}
