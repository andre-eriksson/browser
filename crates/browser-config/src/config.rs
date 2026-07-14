use browser_args::BrowserArgs;
use http::HeaderMap;

use crate::header::Headers;

#[derive(Debug, Clone)]
pub struct BrowserConfig {
    headers: HeaderMap,
}

impl BrowserConfig {
    pub fn new(args: &BrowserArgs) -> Self {
        let headers = Headers::create_browser_headers(args.ua_compatibility, args.user_agent.clone());

        Self { headers }
    }

    #[must_use]
    pub const fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}
