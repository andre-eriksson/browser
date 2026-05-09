use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    sync::{Arc, Mutex, RwLock},
};

use database::{Database, Table};
use rusqlite::{Connection, Result};
use storage::get_data_path;
use time::UtcDateTime;
use tracing::debug;
use url::Host;

use crate::{Expiration, cookie::Cookie, table::CookieTable};

#[derive(Debug)]
pub struct CookieDatabase {
    connection: Mutex<Connection>,
}

impl Database for CookieDatabase {
    fn open() -> Result<Self> {
        let path = get_data_path()
            .ok_or_else(|| rusqlite::Error::InvalidPath("Data path not found".into()))?
            .join("cookies.db");

        std::fs::create_dir_all(path.parent().unwrap())
            .map_err(|_| rusqlite::Error::InvalidPath("Failed to create data directory".into()))?;

        let conn = Connection::open(path)?;

        CookieTable::create_table(&conn)?;

        Ok(Self {
            connection: Mutex::new(conn),
        })
    }
}

#[derive(Debug, Clone)]
pub struct CookieJar {
    inner: Arc<CookieJarInner>,
}

#[derive(Debug)]
pub struct CookieJarInner {
    // TODO: Trie?
    //  top level: com -> google
    //  second level: attributes
    cookies: RwLock<HashMap<Host, Vec<Cookie>>>,
    database: CookieDatabase,
}

impl CookieJar {
    /// Loads existing cookies and returns the cookie jar
    #[must_use]
    pub fn load(database: CookieDatabase) -> Self {
        let mut cookies = HashMap::with_capacity(32);

        if let Ok(conn) = database.connection.lock() {
            cookies = CookieTable::get_all(&conn);
        }

        Self {
            inner: Arc::new(CookieJarInner {
                cookies: RwLock::new(cookies),
                database,
            }),
        }
    }

    #[must_use]
    pub fn get_cookies_for_domain(&self, domain: &str) -> Vec<Cookie> {
        let Ok(host) = Host::parse(domain) else {
            debug!("Invalid domain '{}'", domain);
            return Vec::new();
        };

        let Ok(read) = self.inner.cookies.read() else {
            debug!("Unable to get read lock '{}'", domain);
            return Vec::new();
        };

        match read.get(&host) {
            Some(cookies) => cookies.clone(),
            None => {
                let Ok(conn) = self.inner.database.connection.lock() else {
                    return vec![];
                };

                CookieTable::get_cookies_by_domain(&conn, domain)
            }
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
    /// A vector to the matching stored cookies.
    #[must_use]
    pub fn get_cookies(&self, domain: &Host<&str>, path: &str, secure: bool) -> Vec<Cookie> {
        let cookies = self.get_cookies_for_domain(&domain.to_string());

        cookies
            .into_iter()
            .filter(|cookie| Self::validate_cookie(domain, path, secure, cookie))
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
    pub fn add_cookie(&self, cookie: Cookie, request_domain: Host<String>) {
        if let Some(domain) = cookie.domain()
            && !request_domain
                .to_string()
                .ends_with(domain.to_string().as_str())
        {
            debug!("Cookie rejected: domain '{}' doesn't match request domain '{}'", domain, request_domain);

            return;
        }

        let Ok(mut writer) = self.inner.cookies.write() else {
            debug!("Unable to get write lock");
            return;
        };

        if cookie.max_age().is_none() && *cookie.expires() == Expiration::Session {
            match writer.get_mut(&request_domain) {
                Some(domain_cookies) => domain_cookies.push(cookie),
                None => {
                    let mut domain_cookies = Vec::with_capacity(16);
                    domain_cookies.push(cookie);

                    writer.insert(request_domain, domain_cookies);
                }
            }

            return;
        }

        // TODO: Mark updated cookies as dirty then |
        //                                          v
        // TODO: Scheduler should periodically save cookies
        if let Ok(connection) = self.inner.database.connection.lock()
            && let Err(err) = CookieTable::create_table(&connection)
        {
            debug!("Failed to create cookie table: {}", err);
        }

        if let Some(domain_cookies) = writer.get_mut(&request_domain) {
            domain_cookies.retain(|c| !(c.name() == cookie.name() && c.domain() == cookie.domain()));
            domain_cookies.push(cookie);
        } else {
            let mut domain_cookies = Vec::with_capacity(16);
            domain_cookies.push(cookie);

            writer.insert(request_domain, domain_cookies);
        }
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

impl Display for CookieJar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Ok(read) = self.inner.cookies.read() else {
            debug!("Unable to get read lock");
            return Err(std::fmt::Error);
        };

        for (host, cookies) in read.iter() {
            writeln!(f, "host={}", host)?;

            for cookie in cookies {
                writeln!(f, " - {}={}", cookie.name(), cookie.value())?;
            }
        }
        Ok(())
    }
}
