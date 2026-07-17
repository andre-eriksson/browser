use http::{HeaderValue, header::COOKIE};
use tracing::{debug, trace, trace_span, warn};
use url::Url;

use cookies::{Cookie, CookieJar};
use http_types::request::Request;

const REQUEST_COOKIE: &str = "request_cookie";
const RESPONSE_COOKIE: &str = "response_cookie";
const COOKIE_NAME: &str = "cookie_name";
const COOKIE_VALUE: &str = "cookie_value";

pub fn apply_cookies(request: &mut Request, cookies: &[Cookie]) {
    trace!("Applying {} cookies to request", cookies.len());

    if cookies.is_empty() {
        return;
    }

    let mut cookie_list = Vec::with_capacity(cookies.len());

    for cookie in cookies {
        let cookie_name = cookie.name();
        let cookie_value = cookie.value();

        let span = trace_span!(
            REQUEST_COOKIE,
            { COOKIE_NAME } = %cookie_name,
            { COOKIE_VALUE } = %cookie_value
        );
        let _enter = span.enter();

        trace!("Adding cookie to the request");

        cookie_list.push(format!("{cookie_name}={cookie_value}"))
    }

    let header_value = HeaderValue::try_from(cookie_list.join("; "));

    match header_value {
        Ok(header_value) => {
            request.context.headers.append(COOKIE, header_value);
            trace!("Cookie header added successfully");
        }
        Err(error) => {
            // This should rarely happen since cookies are validated when stored
            warn!(%error, "Failed to create Cookie header");
        }
    }
}

pub fn handle_response_cookie(cookie_jar: &CookieJar, request_url: &Url, header_value: &HeaderValue) {
    let Some(host) = request_url.host() else {
        debug!("Request URL does not have a valid domain host");
        return;
    };

    let Ok(cookie_str) = header_value.to_str() else {
        return;
    };

    let Ok(cookie) = Cookie::parse(cookie_str, request_url) else {
        return;
    };

    let span = trace_span!(
        RESPONSE_COOKIE,
        { COOKIE_NAME } = %cookie.name(),
        { COOKIE_VALUE } = %cookie.value()
    );
    let _enter = span.enter();

    trace!("Storing cookie from response");

    cookie_jar.add_cookie(cookie, host.to_owned());
}
