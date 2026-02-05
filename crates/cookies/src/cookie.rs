use std::{fmt::Display, net::Ipv4Addr};

use crate::errors::CookieParsingError;
use time::{
    Date, Duration, OffsetDateTime, Time, UtcDateTime, UtcOffset, macros::format_description,
};
use url::{Host, Url};

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expiration {
    #[default]
    Session,
    Date(OffsetDateTime),
}

impl Display for Expiration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Session => write!(f, "session"),
            Self::Date(offset) => write!(f, "{}", offset),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum SameSite {
    Strict,
    #[default]
    Lax,
    None,
}

impl From<&str> for SameSite {
    fn from(value: &str) -> Self {
        if value.eq_ignore_ascii_case("strict") {
            SameSite::Strict
        } else if value.eq_ignore_ascii_case("none") {
            SameSite::None
        } else {
            SameSite::Lax
        }
    }
}

impl From<String> for SameSite {
    fn from(value: String) -> Self {
        if value.eq_ignore_ascii_case("strict") {
            SameSite::Strict
        } else if value.eq_ignore_ascii_case("none") {
            SameSite::None
        } else {
            SameSite::Lax
        }
    }
}

impl Display for SameSite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SameSite::Lax => write!(f, "lax"),
            SameSite::Strict => write!(f, "strict"),
            SameSite::None => write!(f, "none"),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Cookie {
    name: Box<str>,
    value: Box<str>,
    expires: Expiration,
    max_age: Option<Duration>,
    domain: Option<Box<Host>>,
    path: Box<str>,
    secure: bool,
    http_only: bool,
    same_site: SameSite,
}

impl Cookie {
    pub fn builder() -> CookieBuilder {
        CookieBuilder::default()
    }

    pub fn parse(cookie_str: &str, request_url: &Url) -> Result<Self, CookieParsingError> {
        let parts = cookie_str.split(';');
        let mut cookie = Cookie::default();

        for part in parts {
            let trimmed = part.trim();

            if trimmed.is_empty() {
                continue;
            }

            if cookie.name.is_empty() {
                let pair = match part.split_once('=') {
                    None => {
                        return Err(CookieParsingError::InvalidCookie);
                    }
                    Some(pair) => pair,
                };

                cookie.name = pair.0.trim().into();
                cookie.value = pair.1.trim().into();

                continue;
            }

            let (k, value) = match trimmed.split_once('=') {
                Some((k, v)) => (k.trim(), Some(v.trim())),
                None => (trimmed, None),
            };

            if k.eq_ignore_ascii_case("expires") {
                Self::parse_expires(&mut cookie, value)?;
            } else if k.eq_ignore_ascii_case("max-age") {
                Self::parse_max_age(&mut cookie, value)?;
            } else if k.eq_ignore_ascii_case("domain") {
                Self::parse_domain(&mut cookie, value)?;
            } else if k.eq_ignore_ascii_case("path") {
                Self::parse_path(&mut cookie, value, request_url);
            } else if k.eq_ignore_ascii_case("samesite") {
                Self::parse_same_site(&mut cookie, value);
            } else if k.eq_ignore_ascii_case("secure") {
                cookie.secure = true;
            } else if k.eq_ignore_ascii_case("httponly") {
                cookie.http_only = true;
            }
        }

        Self::validate_cookie_prefix(&cookie)?;

        Ok(cookie)
    }

    pub(crate) fn validate_cookie_prefix(cookie: &Cookie) -> Result<(), CookieParsingError> {
        if cookie.name().starts_with("__Host-Http-") {
            if !cookie.secure() {
                return Err(CookieParsingError::PrefixMismatch {
                    prefix: String::from("__Host-Http-"),
                    message: String::from("have the Secure attribute"),
                });
            }
            if !cookie.http_only() {
                return Err(CookieParsingError::PrefixMismatch {
                    prefix: String::from("__Host-Http-"),
                    message: String::from("have the HttpOnly attribute"),
                });
            }
            if cookie.domain().is_some() {
                return Err(CookieParsingError::PrefixMismatch {
                    prefix: String::from("__Host-Http-"),
                    message: String::from("not have a Domain attribute"),
                });
            }
            if cookie.path() != "/" {
                return Err(CookieParsingError::PrefixMismatch {
                    prefix: String::from("__Host-Http-"),
                    message: String::from("have Path set to /"),
                });
            }
        } else if cookie.name().starts_with("__Host-") {
            if !cookie.secure() {
                return Err(CookieParsingError::PrefixMismatch {
                    prefix: String::from("__Host-"),
                    message: String::from("have the Secure attribute"),
                });
            }
            if cookie.domain().is_some() {
                return Err(CookieParsingError::PrefixMismatch {
                    prefix: String::from("__Host-"),
                    message: String::from("not have a Domain attribute"),
                });
            }
            if cookie.path() != "/" {
                return Err(CookieParsingError::PrefixMismatch {
                    prefix: String::from("__Host-"),
                    message: String::from("have Path set to /"),
                });
            }
        } else if cookie.name().starts_with("__Http-") {
            if !cookie.secure() {
                return Err(CookieParsingError::PrefixMismatch {
                    prefix: String::from("__Http-"),
                    message: String::from("have the Secure attribute"),
                });
            }
            if !cookie.http_only() {
                return Err(CookieParsingError::PrefixMismatch {
                    prefix: String::from("__Http-"),
                    message: String::from("have the HttpOnly attribute"),
                });
            }
        } else if cookie.name().starts_with("__Secure-") && !cookie.secure() {
            return Err(CookieParsingError::PrefixMismatch {
                prefix: String::from("__Secure-"),
                message: String::from("have the Secure attribute"),
            });
        }

        Ok(())
    }

    fn parse_expires(cookie: &mut Cookie, value: Option<&str>) -> Result<(), CookieParsingError> {
        if let Some(expires) = value {
            let date_parts: Vec<&str> = expires.split_ascii_whitespace().collect();

            if date_parts.len() == 6 {
                // Sun, 06 Nov 1994 08:49:37 GMT
                let date_format = format_description!("[day]-[month repr:short]-[year]");
                let full_date = [date_parts[1], date_parts[2], date_parts[3]].join("-");

                let date = match Date::parse(full_date.as_str(), date_format) {
                    Err(e) => return Err(CookieParsingError::DateError(e.to_string())),
                    Ok(date) => date,
                };

                let time_format = format_description!("[hour]:[minute]:[second]");
                let time = match Time::parse(date_parts[4], time_format) {
                    Err(e) => return Err(CookieParsingError::TimeError(e.to_string())),
                    Ok(parsed) => parsed,
                };

                cookie.expires =
                    Expiration::Date(OffsetDateTime::new_in_offset(date, time, UtcOffset::UTC))
            } else if date_parts.len() == 5 {
                // Sun Nov 6 08:49:37 1994
                let date_format =
                    format_description!("[month repr:short]-[day padding:none]-[year]");
                let full_date = [date_parts[1], date_parts[2], date_parts[4]].join("-");

                let date = match Date::parse(full_date.as_str(), date_format) {
                    Err(e) => return Err(CookieParsingError::DateError(e.to_string())),
                    Ok(date) => date,
                };

                let time_format = format_description!("[hour]:[minute]:[second]");
                let time = match Time::parse(date_parts[3], time_format) {
                    Err(e) => return Err(CookieParsingError::TimeError(e.to_string())),
                    Ok(parsed) => parsed,
                };

                cookie.expires =
                    Expiration::Date(OffsetDateTime::new_in_offset(date, time, UtcOffset::UTC))
            } else if date_parts.len() == 4 {
                // Sunday, 06-Nov-94 08:49:37 GMT
                let correct_date = if date_parts[1][7..].len() == 4 {
                    date_parts[1].to_string()
                } else {
                    let current_year_prefix = UtcDateTime::now()
                        .year()
                        .to_string()
                        .split_at(2)
                        .0
                        .parse::<i16>()
                        .unwrap();

                    format!(
                        "{}{}{}",
                        &date_parts[1][..7],
                        current_year_prefix,
                        &date_parts[1][7..]
                    )
                };

                let date_format = format_description!("[day]-[month repr:short]-[year]");

                let date = match Date::parse(correct_date.trim(), date_format) {
                    Err(e) => return Err(CookieParsingError::DateError(e.to_string())),
                    Ok(date) => date,
                };

                let time_format = format_description!("[hour]:[minute]:[second]");
                let time = match Time::parse(date_parts[2], time_format) {
                    Err(e) => return Err(CookieParsingError::TimeError(e.to_string())),
                    Ok(parsed) => parsed,
                };

                cookie.expires =
                    Expiration::Date(OffsetDateTime::new_in_offset(date, time, UtcOffset::UTC))
            }
        }

        Ok(())
    }

    fn parse_max_age(cookie: &mut Cookie, value: Option<&str>) -> Result<(), CookieParsingError> {
        if let Some(max_age) = value {
            let value = if max_age.starts_with('-') {
                "0"
            } else {
                max_age
            };

            let val = match value.parse::<i64>() {
                Err(e) => {
                    return Err(CookieParsingError::Parsing(
                        String::from("i16"),
                        e.to_string(),
                    ));
                }
                Ok(val) => val,
            };

            let duration = Duration::seconds(val);

            cookie.max_age = Some(duration);
        }

        Ok(())
    }

    fn parse_domain(cookie: &mut Cookie, value: Option<&str>) -> Result<(), CookieParsingError> {
        if let Some(domain) = value {
            let mut domain_mut = domain;

            if domain_mut.starts_with('.') {
                domain_mut = &domain_mut[1..];
            }

            let domain = match Host::parse(domain_mut) {
                Err(e) => {
                    return Err(CookieParsingError::Parsing(
                        String::from("host"),
                        e.to_string(),
                    ));
                }
                Ok(host) => host,
            };

            cookie.domain = Some(domain.into());
        }

        Ok(())
    }

    fn parse_path(cookie: &mut Cookie, value: Option<&str>, request_url: &Url) {
        if let Some(path) = value
            && (path.starts_with('/') || !path.is_empty())
        {
            cookie.path = path.into();
        } else {
            cookie.path = request_url.path().into();
        }
    }

    fn parse_same_site(cookie: &mut Cookie, value: Option<&str>) {
        if let Some(same_site) = value {
            let same_site = if same_site.eq_ignore_ascii_case("strict") {
                SameSite::Strict
            } else if same_site.eq_ignore_ascii_case("none") {
                SameSite::None
            } else {
                SameSite::Lax
            };

            cookie.same_site = same_site
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn expires(&self) -> &Expiration {
        &self.expires
    }

    pub fn max_age(&self) -> &Option<Duration> {
        &self.max_age
    }

    pub fn domain(&self) -> &Option<Box<Host>> {
        &self.domain
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn secure(&self) -> bool {
        self.secure
    }

    pub fn http_only(&self) -> bool {
        self.http_only
    }

    pub fn same_site(&self) -> &SameSite {
        &self.same_site
    }
}

impl Display for Cookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}={}; Expires={}; Max-Age={}; Domain={}; Path={}; Secure={}; HttpOnly={}; SameSite={}",
            self.name(),
            self.value(),
            self.expires(),
            self.max_age().unwrap_or(Duration::seconds(0)),
            self.domain()
                .as_ref()
                .unwrap_or(&Host::Ipv4(Ipv4Addr::new(127, 0, 0, 1)).into()),
            self.path(),
            self.secure(),
            self.http_only(),
            self.same_site()
        )
    }
}

#[derive(Debug, Default, Clone)]
pub struct CookieBuilder {
    name: Box<str>,
    value: Box<str>,
    expires: Expiration,
    max_age: Option<Duration>,
    domain: Option<Box<Host>>,
    path: Box<str>,
    secure: bool,
    http_only: bool,
    same_site: SameSite,
}

impl CookieBuilder {
    pub fn name(mut self, name: String) -> Self {
        self.name = name.into();
        self
    }

    pub fn value(mut self, value: String) -> Self {
        self.value = value.into();
        self
    }

    pub fn expires(mut self, expiration: Expiration) -> Self {
        self.expires = expiration;
        self
    }

    pub fn max_age(mut self, max_age: Duration) -> Self {
        self.max_age = Some(max_age);
        self
    }

    pub fn domain(mut self, domain: Host) -> Self {
        self.domain = Some(domain.into());
        self
    }

    pub fn path(mut self, path: String) -> Self {
        self.path = path.into();
        self
    }

    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = same_site;
        self
    }

    pub fn build(self) -> Result<Cookie, CookieParsingError> {
        let cookie = Cookie {
            name: self.name,
            value: self.value,
            expires: self.expires,
            max_age: self.max_age,
            domain: self.domain,
            path: self.path,
            secure: self.secure,
            http_only: self.http_only,
            same_site: self.same_site,
        };

        Cookie::validate_cookie_prefix(&cookie)?;

        Ok(cookie)
    }

    /// Builds a cookie without validating the prefix. Use with caution, as this may result in cookies that do not adhere to the expected security and formatting rules.
    pub fn build_unchecked(self) -> Cookie {
        Cookie {
            name: self.name,
            value: self.value,
            expires: self.expires,
            max_age: self.max_age,
            domain: self.domain,
            path: self.path,
            secure: self.secure,
            http_only: self.http_only,
            same_site: self.same_site,
        }
    }
}
