use std::{
    fmt::{Display, Formatter},
    net::Ipv4Addr,
};

use database::{Database, Domain, Table};
use tracing::debug;
use url::Host;

use crate::{Expiration, cookie::Cookie, table::CookieTable};

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
#[derive(Clone)]
pub struct CookieJar {
    /// The list of stored cookies.
    session_cookies: Vec<Cookie>,

    database: Database,
}

impl Default for CookieJar {
    fn default() -> Self {
        Self {
            session_cookies: Vec::new(),
            database: Database::new(Domain::Cookies),
        }
    }
}

impl CookieJar {
    /// Creates a new, empty CookieJar.
    pub fn new() -> Self {
        CookieJar {
            session_cookies: Vec::with_capacity(16),
            database: Database::new(Domain::Cookies),
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
    pub fn get_cookies(&self, domain: &str, path: &str, secure: bool) -> Vec<Cookie> {
        let mut result = Vec::new();

        let host = match Host::parse(domain) {
            Err(e) => {
                eprintln!("{e}");
                return result;
            }
            Ok(h) => h,
        };

        let conn = self.database.open();
        if let Ok(connection) = conn {
            let persisted_cookies = CookieTable::get_cookies_by_domain(&connection, domain);

            result.extend(persisted_cookies);
        }

        for cookie in self.session_cookies.clone() {
            if let Some(cookie_domain) = cookie.domain() {
                if cookie_domain != &host {
                    continue;
                }
            } else if let Some(domain) = cookie.domain()
                && domain != &host
            {
                continue;
            }

            if cookie.secure() && !secure {
                continue;
            }

            if !path.starts_with(cookie.path()) {
                continue;
            }

            result.push(cookie);
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
    pub fn add_cookie(&mut self, cookie: Cookie, request_domain: Host) {
        if let Some(domain) = cookie.domain()
            && !request_domain
                .to_string()
                .ends_with(domain.to_string().as_str())
        {
            debug!(
                "Cookie rejected: domain '{}' doesn't match request domain '{}'",
                domain, request_domain
            );

            return;
        }

        // TODO: Handle age/expiration, max cookies

        if cookie.max_age().is_none() && cookie.expires() == &Expiration::Session {
            self.session_cookies.push(cookie);
            return;
        }

        let conn = self.database.open();

        if let Ok(connection) = conn {
            let creation = CookieTable::create_table(&connection);

            if creation.is_err() {
                debug!(
                    "Unable to create the cookie table: {}",
                    creation.err().unwrap()
                );
            } else {
                let adding = CookieTable::insert(&connection, &cookie);

                if adding.is_err() {
                    debug!("Unable to add a cookie: {}", adding.err().unwrap());
                }
            }
        }
    }
}

impl Iterator for CookieJar {
    type Item = Cookie;

    fn next(&mut self) -> Option<Self::Item> {
        if self.session_cookies.is_empty() {
            None
        } else {
            Some(self.session_cookies.remove(0))
        }
    }
}

impl Display for CookieJar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for cookie in &self.session_cookies {
            writeln!(f, "{}={}", cookie.name(), cookie.value())?;
        }
        Ok(())
    }
}
