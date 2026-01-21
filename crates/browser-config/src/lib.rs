use constants::files::SETTINGS;
use serde::{Deserialize, Serialize};
use storage::{
    files::read_file_from_config,
    paths::{create_paths, get_config_path},
};

// RGB Value
pub type Color = [u8; 3];

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
            background: [255, 255, 255],
            foreground: [216, 216, 209],
            text: [0, 0, 0],
            primary: [144, 230, 252],
            success: [18, 102, 79],
            warning: [183, 126, 51],
            danger: [195, 66, 63],
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
