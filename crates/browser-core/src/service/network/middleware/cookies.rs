use std::sync::RwLock;

use cookies::{Cookie, CookieJar};
use http::{HeaderValue, header::COOKIE};
use network::http::request::Request;
use telemetry::keys::{COOKIE_NAME, COOKIE_VALUE, REQUEST_COOKIE, RESPONSE_COOKIE};
use tracing::{debug, trace, trace_span, warn};
use url::Host;

pub struct CookieMiddleware;

impl CookieMiddleware {
    /// Applies cookies from the cookie jar to the given request.
    ///
    /// # Arguments
    /// * `request` - The HTTP request to which cookies will be applied.
    /// * `cookie_jar` - The cookie jar containing stored cookies.
    ///
    /// # Notes
    /// This function modifies the `request` in place by adding the appropriate Cookie headers.
    pub fn apply_cookies(request: &mut Request, cookie_jar: &RwLock<CookieJar>) {
        let cookies = if let Ok(jar) = cookie_jar.read() {
            jar.get_cookies(
                request.url.host().unwrap(),
                request.url.path(),
                request.url.scheme() == "https",
            )
        } else {
            Vec::new()
        };

        trace!("Applying {} cookies to request", cookies.len());

        for cookie in cookies {
            let cookie_name = cookie.name();
            let cookie_value = cookie.value();

            let span = trace_span!(
                REQUEST_COOKIE,
                { COOKIE_NAME } = %cookie_name,
                { COOKIE_VALUE } = %cookie_value
            );

            let _enter = span.enter();

            let cookie_str = format!("{}={}", cookie_name, cookie_value);

            let cookie_header = HeaderValue::from_str(&cookie_str);

            match cookie_header {
                Ok(header_value) => {
                    request.headers.append(COOKIE, header_value);
                    trace!("Cookie header added successfully");
                }
                Err(e) => {
                    // This should rarely happen since cookies are validated when stored
                    warn!("Failed to create Cookie header for '{}': {}", cookie_str, e);
                    continue;
                }
            }
        }
    }

    /// Handles a Set-Cookie header from an HTTP response and stores the cookie in the cookie jar.
    ///
    /// # Arguments
    /// * `cookie_jar` - The cookie jar where the cookie will be stored.
    /// * `request_domain` - The domain of the request that received the response.
    /// * `header_value` - The value of the Set-Cookie header from the response.
    pub fn handle_response_cookie(
        cookie_jar: &mut RwLock<CookieJar>,
        request_domain: Host,
        header_value: &HeaderValue,
    ) {
        let cookie_str = match header_value.to_str() {
            Ok(s) => s,
            Err(_) => return,
        };

        let cookie = match Cookie::parse(cookie_str) {
            Ok(c) => c,
            Err(e) => {
                debug!("Error parsing the cookie: {e}");
                return;
            }
        };

        let span = trace_span!(
            RESPONSE_COOKIE,
            { COOKIE_NAME } = %cookie.name(),
            { COOKIE_VALUE } = %cookie.value()
        );
        let _enter = span.enter();

        trace!("Storing cookie from response");

        if let Ok(jar) = cookie_jar.get_mut() {
            jar.add_cookie(cookie, request_domain);
        }
    }
}
