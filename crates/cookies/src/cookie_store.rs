use std::fmt::{Display, Formatter};

use database::{Database, Domain, Table};
use time::UtcDateTime;
use tracing::debug;
use url::Host;

use crate::{Expiration, cookie::Cookie, table::CookieTable};

/// A simple in-memory cookie jar (for now).
#[derive(Clone)]
pub struct CookieJar {
    /// The list of stored cookies.
    cookies: Vec<Cookie>,

    /// The database instance for cookies
    database: Database,
}

impl Default for CookieJar {
    fn default() -> Self {
        Self {
            cookies: Vec::new(),
            database: Database::new(Domain::Cookies),
        }
    }
}

impl CookieJar {
    /// Loads existing cookies and returns the cookie jar
    pub fn load() -> Self {
        let database = Database::new(Domain::Cookies);

        let conn = database.open();
        if let Ok(connection) = conn {
            let cookies = CookieTable::get_all(&connection);

            return Self { database, cookies };
        }

        CookieJar {
            cookies: Vec::with_capacity(32),
            database: Database::new(Domain::Cookies),
        }
    }

    pub fn cookies(&self) -> &Vec<Cookie> {
        &self.cookies
    }

    pub fn get_cookies_for_domain(&self, domain: &str) -> Vec<Cookie> {
        let host = Host::parse(domain);
        if host.is_err() {
            return Vec::new();
        }

        self.cookies
            .iter()
            .filter(|cookie| {
                if let Some(cookie_domain) = cookie.domain() {
                    **cookie_domain == *host.as_ref().unwrap()
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// Retrieves cookies that match the given domain, path, and security context.
    ///
    /// # Arguments
    /// * `domain` - The domain to match against.
    /// * `path` - The path to match against.
    /// * `secure` - Whether the request is made over a secure connection.
    ///
    /// # Returns
    /// A vector to the matching stored cookies.
    pub fn get_cookies(&self, domain: Host<&str>, path: &str, secure: bool) -> Vec<Cookie> {
        let mut cookies = self.cookies.clone();

        let conn = self.database.open();
        if let Ok(connection) = conn {
            let persisted_cookies =
                CookieTable::get_cookies_by_domain(&connection, domain.to_string());

            cookies.extend(persisted_cookies);
        }

        cookies
            .into_iter()
            .filter(|cookie| Self::validate_cookie(&domain, path, secure, cookie))
            .collect()
    }

    /// Adds a cookie to the jar if it matches the request domain.
    ///
    /// # Arguments
    /// * `cookie` - The cookie to add.
    /// * `request_domain` - The domain of the request setting the cookie.
    ///
    /// # Notes
    /// This function currently does not handle cookie expiration or maximum cookie limits.
    pub fn add_cookie(&mut self, cookie: Cookie, request_domain: Host<&str>) {
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

        if cookie.max_age().is_none() && cookie.expires() == &Expiration::Session {
            self.cookies.push(cookie);
            return;
        }

        // TODO: Mark updated cookies as dirty then |
        //                                          v
        // TODO: Scheduler should periodically save cookies
        if let Ok(connection) = self.database.open() {
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

        if let Some(pos) = self
            .cookies
            .iter()
            .position(|c| c.name() == cookie.name() && c.domain() == cookie.domain())
        {
            self.cookies.remove(pos);
        }

        self.cookies.push(cookie);
    }

    fn validate_cookie(domain: &Host<&str>, path: &str, secure: bool, cookie: &Cookie) -> bool {
        if let Some(cookie_domain) = cookie.domain()
            && **cookie_domain != *domain
        {
            return false;
        }

        if cookie.secure() && !secure {
            return false;
        }

        if !path.starts_with(cookie.path().trim()) {
            return false;
        }

        if let Some(max_age) = cookie.max_age() {
            let current_time = UtcDateTime::now().unix_timestamp();
            if max_age.whole_seconds() <= current_time {
                // TODO: Mark as stale/remove them from the database
                return false;
            }
        }

        true
    }
}

impl Iterator for CookieJar {
    type Item = Cookie;

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
        for cookie in &self.cookies {
            writeln!(f, "{}={}", cookie.name(), cookie.value())?;
        }
        Ok(())
    }
}
