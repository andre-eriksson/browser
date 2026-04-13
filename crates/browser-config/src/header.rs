use manifest::{APP_NAME, APP_VERSION};
use network::{
    ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CACHE_CONTROL, CONNECTION, HeaderMap, HeaderValue, USER_AGENT,
};
use sys_locale::get_locales;
use tracing::error;

pub struct Headers;

impl Headers {
    pub(crate) fn create_browser_headers(compatibility: bool, custom_user_agent: Option<String>) -> HeaderMap {
        let mut browser_headers = HeaderMap::with_capacity(6);
        let user_agent = custom_user_agent.unwrap_or_else(|| Self::get_user_agent(compatibility));

        macro_rules! insert_header {
            ($name:ident, $value:expr) => {
                match HeaderValue::from_bytes($value) {
                    Ok(header_value) => {
                        browser_headers.insert($name, header_value);
                    }
                    Err(e) => {
                        error!("Failed to create header value for {}: {}", stringify!($name), e);
                    }
                }
            };
        }

        insert_header!(ACCEPT, b"text/html,*/*;q=0.8");
        insert_header!(ACCEPT_ENCODING, b"gzip, deflate, br");
        insert_header!(ACCEPT_LANGUAGE, Self::get_accept_language_value().as_bytes());
        insert_header!(CACHE_CONTROL, b"no-store");
        insert_header!(CONNECTION, b"keep-alive");
        insert_header!(USER_AGENT, user_agent.as_bytes());

        browser_headers
    }

    fn get_accept_language_value() -> String {
        let locales = get_locales().collect::<Vec<_>>();

        if locales.is_empty() {
            return "en-US,en;q=0.9".to_string();
        }

        let mut accept_language = String::new();
        for (i, locale) in locales.iter().enumerate() {
            if i == 0 {
                accept_language.push_str(&format!("{},", locale));
            } else {
                accept_language.push_str(&format!("{};q={:.1},", locale, (i as f32).mul_add(-0.1, 0.9)));
            }
        }

        accept_language.trim_end_matches(',').to_string()
    }

    fn get_user_agent(compatibility: bool) -> String {
        #[cfg(target_os = "windows")]
        let os_info = "Windows NT 10.0; Win64; x64";
        #[cfg(target_os = "linux")]
        let os_info = "X11; Linux x86_64";
        #[cfg(target_os = "macos")]
        let os_info = "Macintosh; Intel Mac OS X 10_15_7";

        match compatibility {
            true => {
                format!("Mozilla/5.0 ({os_info}) AppleWebKit/537.36 (KHTML, like Gecko) {APP_NAME}/{APP_VERSION}")
            }
            false => format!("Mozilla/5.0 ({os_info}) {APP_NAME}/{APP_VERSION}"),
        }
    }
}
