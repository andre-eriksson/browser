use browser_preferences::BrowserPreferences;
use clap::Parser;
use network::{HeaderMap, HeaderName, HeaderValue};

use crate::{
    args::BrowserArgs,
    header::{HeaderType, Headers},
};

#[derive(Debug, Clone)]
pub struct BrowserConfig {
    args: BrowserArgs,
    headers: HeaderMap,
    preferences: BrowserPreferences,
}

impl BrowserConfig {
    pub fn new() -> Self {
        let args = BrowserArgs::parse();

        let mut headers = match args.headless {
            true => Headers::create_browser_headers(HeaderType::HeadlessBrowser),
            false => Headers::create_browser_headers(HeaderType::Browser),
        };

        for header in args.headers.iter() {
            if let Some((key, value)) = header.split_once(':')
                && let Ok(header_name) = HeaderName::from_bytes(key.trim().as_bytes())
                && let Ok(header_value) = HeaderValue::from_str(value.trim())
            {
                headers.insert(header_name, header_value);
            }
        }

        let preferences = if let Some(theme) = &args.theme {
            BrowserPreferences::new(theme.clone())
        } else {
            BrowserPreferences::load()
        };

        Self {
            args,
            headers,
            preferences,
        }
    }

    pub fn args(&self) -> &BrowserArgs {
        &self.args
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn preferences(&self) -> &BrowserPreferences {
        &self.preferences
    }
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self::new()
    }
}
