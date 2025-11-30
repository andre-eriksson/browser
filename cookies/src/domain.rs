/// Determines if a cookie's domain matches a request's domain according to standard cookie rules.
///
/// # Arguments
/// * `cookie_domain` - The domain attribute of the cookie.
/// * `request_domain` - The domain of the request.
///
/// # Returns
/// * `true` if the cookie domain matches the request domain, `false` otherwise.
pub fn domain_matches(cookie_domain: &str, request_domain: &str) -> bool {
    let request = request_domain.to_lowercase();
    let cookie = cookie_domain.to_lowercase();

    if request == cookie {
        return true;
    }

    if request.ends_with(&cookie) {
        let prefix_len = request.len() - cookie.len();
        if prefix_len > 0 && request.as_bytes()[prefix_len - 1] == b'.' {
            return true;
        }
    }

    if cookie.starts_with('.') && request.ends_with(&cookie[1..]) {
        return true;
    }

    false
}
