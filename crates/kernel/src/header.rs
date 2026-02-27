use std::collections::HashMap;

use constants::APP_NAME;
use network::{
    ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CACHE_CONTROL, CONNECTION, HeaderMap, HeaderValue, USER_AGENT,
};

pub struct DefaultHeaders;

pub enum HeaderType {
    Browser,
    HeadlessBrowser,
}

impl DefaultHeaders {
    pub fn get_user_agent(browser_type: HeaderType) -> String {
        #[cfg(target_os = "windows")]
        let os_info = "Windows NT 10.0; Win64; x64";
        #[cfg(target_os = "linux")]
        let os_info = "X11; Linux x86_64";
        #[cfg(target_os = "macos")]
        let os_info = "Macintosh; Intel Mac OS X 10_15_7";

        let mut user_agent = match browser_type {
            HeaderType::Browser => {
                format!("Mozilla/5.0 ({}) MyRenderer/0.1 {}/0.1", os_info, APP_NAME)
            }
            HeaderType::HeadlessBrowser => format!("Mozilla/5.0 ({}) MyRendererHeadless/0.1 {}/0.1", os_info, APP_NAME),
        };

        #[cfg(debug_assertions)]
        {
            user_agent += "-development (contact: https://github.com/andre-eriksson/browser/issues)";
        }

        user_agent
    }

    pub fn create_browser_headers(browser_type: HeaderType) -> HeaderMap {
        let mut browser_headers = HeaderMap::new();

        let mut headers = HashMap::new();
        headers.insert(ACCEPT, "text/html".to_string());
        headers.insert(ACCEPT_ENCODING, "gzip, deflate, br".to_string());
        headers.insert(ACCEPT_LANGUAGE, "en-US, sv-SE;q=0.9, en;q=0.8, sv;q=0.7".to_string());
        headers.insert(CACHE_CONTROL, "no-store".to_string());
        headers.insert(CONNECTION, "keep-alive".to_string());

        let user_agent = Self::get_user_agent(browser_type);
        headers.insert(USER_AGENT, user_agent);

        for (key, value) in headers {
            if let Ok(header_value) = HeaderValue::from_str(&value) {
                browser_headers.insert(key, header_value);
            } else {
                eprintln!("Failed to create header value for {}: {}", key.as_str(), value);
            }
        }

        browser_headers
    }
}
