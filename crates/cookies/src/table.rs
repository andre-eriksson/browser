use std::panic;

use database::Table;
use rusqlite::{Connection, Result, params};
use time::{Duration, OffsetDateTime, UtcDateTime};
use url::Host;

use crate::{Cookie, Expiration, cookie::SameSite};

pub struct CookieTable;

impl CookieTable {
    /// Retrieves all cookies
    pub fn get_all(conn: &Connection) -> Vec<Cookie> {
        let mut cookies = Vec::with_capacity(32);

        let stmt =
            conn.prepare("SELECT name, value, expiration, domain, path, secure, http_only, same_site FROM cookies");

        if stmt.is_err() {
            return cookies;
        }

        let mut binding = stmt.unwrap();
        let cookies_iter = binding.query_map([], |row| {
            let val = row.get::<usize, i64>(2);

            let Ok(expiry) = OffsetDateTime::from_unix_timestamp(val?) else {
                panic!();
            };

            let dom_host = row.get::<usize, String>(3)?;
            let Ok(domain) = Host::parse(dom_host.as_str()) else {
                panic!();
            };

            let cookie = Cookie::builder()
                .name(row.get(0)?)
                .value(row.get(1)?)
                .expires(Expiration::Date(expiry))
                .domain(domain)
                .path(row.get(4)?)
                .secure(row.get(5)?)
                .http_only(row.get(6)?)
                .same_site(SameSite::from(row.get::<usize, String>(7)?))
                .build_unchecked();

            Ok(cookie)
        });

        if cookies_iter.is_err() {
            return cookies;
        }

        for cookie in cookies_iter.unwrap() {
            if cookie.is_err() {
                continue;
            }

            cookies.push(cookie.unwrap());
        }

        cookies
    }

    /// Retrieve all cookies from a specific domain
    pub fn get_cookies_by_domain<D: AsRef<str>>(conn: &Connection, domain: D) -> Vec<Cookie> {
        let mut cookies = Vec::new();

        let stmt = conn.prepare(
            "SELECT name, value, expiration, domain, path, secure, http_only, same_site FROM cookies WHERE domain=?1",
        );

        if stmt.is_err() {
            return cookies;
        }

        let mut binding = stmt.unwrap();
        let cookies_iter = binding.query_map([domain.as_ref()], |row| {
            let val = row.get::<usize, i64>(2)?;

            let Ok(expiry) = OffsetDateTime::from_unix_timestamp(val) else {
                panic!();
            };

            let dom_host = row.get::<usize, String>(3)?;
            let Ok(domain) = Host::parse(dom_host.as_str()) else {
                panic!();
            };

            let cookie = Cookie::builder()
                .name(row.get(0)?)
                .value(row.get(1)?)
                .expires(Expiration::Date(expiry))
                .max_age(Duration::seconds(val))
                .domain(domain)
                .path(row.get(4)?)
                .secure(row.get(5)?)
                .secure(row.get(6)?)
                .same_site(SameSite::from(row.get::<usize, String>(7)?))
                .build_unchecked();

            Ok(cookie)
        });

        if cookies_iter.is_err() {
            return cookies;
        }

        for cookie in cookies_iter.unwrap() {
            if cookie.is_err() {
                continue;
            }

            cookies.push(cookie.unwrap());
        }

        cookies
    }
}

impl Table for CookieTable {
    type Record = Cookie;

    fn create_table(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "BEGIN;
            CREATE TABLE IF NOT EXISTS cookies (
                id INTEGER PRIMARY KEY,
                name TEXT,
                value TEXT,
                expiration INTEGER,
                domain TEXT,
                path TEXT,
                secure BOOLEAN,
                http_only BOOLEAN,
                same_site TEXT,
                UNIQUE (name, domain)
            );
            CREATE INDEX IF NOT EXISTS domain_idx ON cookies (domain);
            COMMIT;",
        )?; // TODO: index on expiration/max-age
        Ok(())
    }

    fn insert(conn: &Connection, data: &Self::Record) -> Result<()> {
        let expiry = match data.expires() {
            Expiration::Session => None,
            Expiration::Date(offset) => Some(offset.unix_timestamp()),
        };

        let max_age = match data.max_age() {
            None => expiry.unwrap_or_default(),
            Some(val) => UtcDateTime::now().unix_timestamp() + val.whole_seconds(),
        };

        let same_site = data.same_site().to_string();

        conn.execute(
            "INSERT OR REPLACE INTO cookies
            (name, value, expiration, domain, path, secure, http_only, same_site)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                data.name(),
                data.value(),
                max_age,
                data.domain()
                    .as_ref()
                    .map(|h| h.to_string())
                    .unwrap_or_else(|| "127.0.0.1".to_string()),
                data.path(),
                data.secure(),
                data.http_only(),
                same_site,
            ],
        )?;

        Ok(())
    }
}
