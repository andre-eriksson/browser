use std::{
    fmt::{Display, Formatter},
    net::Ipv4Addr,
};

use tracing::debug;
use url::Host;

use crate::cookie::Cookie;

/// A stored cookie with additional metadata.
#[derive(Clone, Debug)]
pub struct StoredCookie {
    /// The actual cookie data.
    pub inner: Cookie,

    /// The original host from which the cookie was set (if host-only), will be None for domain cookies.
    original_host: Option<Host>,
}

impl Display for StoredCookie {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}={}; Domain={}; Path={}; Secure={}; HttpOnly={}",
            self.inner.name(),
            self.inner.value(),
            self.inner.domain().as_ref().unwrap_or(
                &self
                    .original_host
                    .clone()
                    .unwrap_or(Host::Ipv4(Ipv4Addr::new(127, 0, 0, 1)))
            ),
            self.inner.path(),
            self.inner.secure(),
            self.inner.http_only(),
        )
    }
}

/// A simple in-memory cookie jar (for now).
#[derive(Default, Clone)]
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

        let host = match Host::parse(domain) {
            Err(e) => {
                eprintln!("{e}");
                return result;
            }
            Ok(h) => h,
        };

        for stored_cookie in &self.cookies {
            if let Some(cookie_domain) = stored_cookie.inner.domain() {
                if cookie_domain != &host {
                    continue;
                }
            } else if let Some(original_host) = &stored_cookie.original_host
                && original_host != &host
            {
                continue;
            }

            if stored_cookie.inner.secure() && !secure {
                continue;
            }

            if !path.starts_with(stored_cookie.inner.path()) {
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
    pub fn add_cookie(&mut self, cookie: Cookie, request_domain: &str) {
        let request_host = match Host::parse(request_domain) {
            Err(e) => {
                eprintln!("{e}");
                return;
            }
            Ok(h) => h,
        };

        if let Some(domain) = cookie.domain()
            && domain != &request_host
        {
            debug!(
                "Cookie rejected: domain '{}' doesn't match request domain '{}'",
                domain, request_domain
            );

            return;
        }

        // TODO: Handle age/expiration, max cookies

        let original_host = if cookie.domain().is_none() {
            Some(request_host)
        } else {
            None
        };

        self.cookies.push(StoredCookie {
            inner: cookie,
            original_host,
        });
    }
}

impl Iterator for CookieJar {
    type Item = StoredCookie;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cookies.is_empty() {
            None
        } else {
            Some(self.cookies.remove(0))
        }
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
