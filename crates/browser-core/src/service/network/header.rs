use std::collections::HashMap;

use constants::APP_NAME;
use http::{
    HeaderMap, HeaderValue,
    header::{ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CACHE_CONTROL, CONNECTION, USER_AGENT},
};

pub struct DefaultHeaders;

impl DefaultHeaders {
    pub fn create_default_browser_headers() -> HeaderMap {
        let mut browser_headers = HeaderMap::new();

        let mut headers = HashMap::new();
        headers.insert(ACCEPT, "text/html".to_string());
        headers.insert(ACCEPT_ENCODING, "gzip, deflate, br".to_string());
        headers.insert(
            ACCEPT_LANGUAGE,
            "en-US, sv-SE;q=0.9, en;q=0.8, sv;q=0.7".to_string(),
        );
        headers.insert(CACHE_CONTROL, "no-store".to_string());
        headers.insert(CONNECTION, "keep-alive".to_string());

        let user_agent = format!(
            "Mozilla/5.0 (X11; Linux x86_64; rv:0.1) MyRenderer/0.1 {}/0.1",
            APP_NAME
        );
        headers.insert(USER_AGENT, user_agent);

        for (key, value) in headers {
            if let Ok(header_value) = HeaderValue::from_str(&value) {
                browser_headers.insert(key, header_value);
            } else {
                eprintln!(
                    "Failed to create header value for {}: {}",
                    key.as_str(),
                    value
                );
            }
        }

        browser_headers
    }

    pub fn create_headless_browser_headers() -> HeaderMap {
        let mut browser_headers = HeaderMap::new();

        let mut headers = HashMap::new();
        headers.insert(ACCEPT, "text/html".to_string());
        headers.insert(ACCEPT_ENCODING, "gzip, deflate, br".to_string());
        headers.insert(
            ACCEPT_LANGUAGE,
            "en-US, sv-SE;q=0.9, en;q=0.8, sv;q=0.7".to_string(),
        );
        headers.insert(CACHE_CONTROL, "no-store".to_string());
        headers.insert(CONNECTION, "keep-alive".to_string());

        let user_agent = format!(
            "Mozilla/5.0 (X11; Linux x86_64; rv:0.1) Headless/0.1 {}/0.1",
            APP_NAME
        );
        headers.insert(USER_AGENT, user_agent);

        for (key, value) in headers {
            if let Ok(header_value) = HeaderValue::from_str(&value) {
                browser_headers.insert(key, header_value);
            } else {
                eprintln!(
                    "Failed to create header value for {}: {}",
                    key.as_str(),
                    value
                );
            }
        }

        browser_headers
    }
}
