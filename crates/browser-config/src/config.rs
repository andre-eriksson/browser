use browser_preferences::BrowserPreferences;
use clap::Parser;
use network::HeaderMap;

use crate::{args::BrowserArgs, header::Headers};

#[derive(Debug, Clone)]
pub struct BrowserConfig {
    args: BrowserArgs,
    headers: HeaderMap,
    preferences: BrowserPreferences,
}

impl BrowserConfig {
    pub fn new() -> Self {
        let args = BrowserArgs::parse();
        let headers = Headers::create_browser_headers(args.ua_compatibility, args.user_agent.clone());
        let preferences = args
            .theme
            .as_ref()
            .map(|t| BrowserPreferences::new(t.clone()))
            .unwrap_or_else(BrowserPreferences::load);

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
