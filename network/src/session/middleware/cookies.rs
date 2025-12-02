use cookies::cookie_store::CookieJar;
use http::{HeaderValue, header::COOKIE};

use crate::http::request::Request;

/// Applies cookies from the cookie jar to the given request.
///
/// # Arguments
/// * `request` - The HTTP request to which cookies will be applied.
/// * `cookie_jar` - The cookie jar containing stored cookies.
///
/// # Notes
/// This function modifies the `request` in place by adding the appropriate Cookie headers.
pub fn apply_cookies(request: &mut Request, cookie_jar: &CookieJar) {
    let Some(domain) = request.url.domain() else {
        return;
    };

    let secure = request.url.scheme() == "https";

    let cookies = cookie_jar.get_cookies(domain, request.url.path(), secure);

    for stored_cookie in cookies {
        let cookie_str = format!(
            "{}={}",
            stored_cookie.inner.name(),
            stored_cookie.inner.value()
        );

        let cookie_header = HeaderValue::from_str(&cookie_str);

        match cookie_header {
            Ok(header_value) => {
                request.headers.append(COOKIE, header_value);
            }
            Err(_) => {
                // TODO: Logging, realistically this should never happen, cause we parse cookies when storing them
                continue;
            }
        }
    }
}
