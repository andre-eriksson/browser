use std::fmt::{Display, Formatter};

use cookie::Cookie;

use crate::domain::domain_matches;

/// A stored cookie with additional metadata.
pub struct StoredCookie {
    /// The actual cookie data.
    pub inner: Cookie<'static>,

    /// The original host from which the cookie was set (if host-only), will be None for domain cookies.
    original_host: Option<String>,
}

/// A simple in-memory cookie jar (for now).
#[derive(Default)]
pub struct CookieJar {
    /// The list of stored cookies.
    cookies: Vec<StoredCookie>,
}

impl CookieJar {
    /// Creates a new, empty CookieJar.
    pub fn new() -> Self {
        CookieJar {
            cookies: Vec::new(),
        }
    }

    /// Retrieves cookies that match the given domain, path, and security context.
    ///
    /// # Arguments
    /// * `domain` - The domain to match against.
    /// * `path` - The path to match against.
    /// * `secure` - Whether the request is made over a secure connection.
    ///
    /// # Returns
    /// A vector of references to the matching stored cookies.
    pub fn get_cookies(&self, domain: &str, path: &str, secure: bool) -> Vec<&StoredCookie> {
        let mut result = Vec::new();

        for stored_cookie in &self.cookies {
            if let Some(cookie_domain) = stored_cookie.inner.domain() {
                if !domain_matches(cookie_domain, domain) {
                    continue;
                }
            } else if let Some(original_host) = &stored_cookie.original_host
                && original_host != domain
            {
                continue;
            }

            if let Some(cookie_secure) = stored_cookie.inner.secure()
                && cookie_secure
                && !secure
            {
                continue;
            }

            if let Some(cookie_path) = stored_cookie.inner.path()
                && !path.starts_with(cookie_path)
            {
                continue;
            }

            result.push(stored_cookie);
        }

        result
    }

    /// Adds a cookie to the jar if it matches the request domain.
    ///
    /// # Arguments
    /// * `cookie` - The cookie to add.
    /// * `request_domain` - The domain of the request setting the cookie.
    ///
    /// # Notes
    /// This function currently does not handle cookie expiration or maximum cookie limits.
    pub fn add_cookie(&mut self, cookie: Cookie<'static>, request_domain: &str) {
        if let Some(domain) = cookie.domain() {
            if !domain_matches(domain, request_domain) {
                println!(
                    "Cookie rejected: domain '{}' doesn't match request domain '{}'",
                    domain, request_domain
                );
                return;
            }
        }

        // TODO: Handle age/expiration, max cookies

        let host_only = cookie.domain().is_none();
        let original_host = if host_only {
            Some(request_domain.to_string())
        } else {
            None
        };

        self.cookies.push(StoredCookie {
            inner: cookie,
            original_host,
        });
    }
}

impl Display for CookieJar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for stored_cookie in &self.cookies {
            writeln!(
                f,
                "{}={}",
                stored_cookie.inner.name(),
                stored_cookie.inner.value()
            )?;
        }
        Ok(())
    }
}
