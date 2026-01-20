use std::{fmt::Display, net::Ipv4Addr};

use errors::parsing::CookieParsingError;
use time::{
    Date, Duration, OffsetDateTime, Time, UtcDateTime, UtcOffset, macros::format_description,
};
use url::Host;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expiration {
    #[default]
    Session,
    Date(OffsetDateTime),
}

impl Display for Expiration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Self::Session => String::from("session"),
            Self::Date(offset) => offset.to_string(),
        };

        write!(f, "{val}")
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
        match value {
            "strict" => SameSite::Strict,
            "lax" => SameSite::Lax,
            "none" => SameSite::None,
            _ => SameSite::Strict,
        }
    }
}

impl From<String> for SameSite {
    fn from(value: String) -> Self {
        SameSite::from(value.to_ascii_lowercase().as_str())
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

#[derive(Debug, Default, Clone)]
pub struct Cookie {
    name: String,
    value: String,
    expires: Expiration,
    max_age: Option<Duration>,
    domain: Option<Host>,
    path: String,
    secure: bool,
    http_only: bool,
    same_site: SameSite,
}

impl Cookie {
    pub fn builder() -> CookieBuilder {
        CookieBuilder::default()
    }

    pub fn parse(cookie_str: &str) -> Result<Self, CookieParsingError> {
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

                cookie.name = String::from(pair.0.trim());
                cookie.value = String::from(pair.1.trim());

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
                Self::parse_path(&mut cookie, value);
            } else if k.eq_ignore_ascii_case("samesite") {
                Self::parse_same_site(&mut cookie, value);
            } else if k.eq_ignore_ascii_case("secure") {
                cookie.secure = true;
            } else if k.eq_ignore_ascii_case("httponly") {
                cookie.http_only = true;
            }
        }

        Ok(cookie)
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
                // TODO: Something?
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

            cookie.domain = Some(domain);
        }

        Ok(())
    }

    fn parse_path(cookie: &mut Cookie, value: Option<&str>) {
        if let Some(path) = value
            && (path.starts_with('/') || !path.is_empty())
        {
            cookie.path = String::from(path);
            // TODO: Handle "default-path"
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

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn value(&self) -> &String {
        &self.value
    }

    pub fn expires(&self) -> &Expiration {
        &self.expires
    }

    pub fn max_age(&self) -> &Option<Duration> {
        &self.max_age
    }

    pub fn domain(&self) -> &Option<Host> {
        &self.domain
    }

    pub fn path(&self) -> &String {
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
                .unwrap_or(&Host::Ipv4(Ipv4Addr::new(127, 0, 0, 1))),
            self.path(),
            self.secure(),
            self.http_only(),
            self.same_site()
        )
    }
}

#[derive(Debug, Default, Clone)]
pub struct CookieBuilder {
    name: String,
    value: String,
    expires: Expiration,
    max_age: Option<Duration>,
    domain: Option<Host>,
    path: String,
    secure: bool,
    http_only: bool,
    same_site: SameSite,
}

impl CookieBuilder {
    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn value(mut self, value: String) -> Self {
        self.value = value;
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
        self.domain = Some(domain);
        self
    }

    pub fn path(mut self, path: String) -> Self {
        self.path = path;
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

    pub fn build(self) -> Cookie {
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

#[cfg(test)]
mod tests {
    use time::Duration;
    use url::Host;

    use crate::{cookie::Cookie, cookie::Expiration};

    #[test]
    fn expires_format_1() {
        let cookie = Cookie::parse("ID=HelloWorld; Expires=Sun, 06 Nov 1994 08:49:37 GMT").unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert_ne!(cookie.expires(), &Expiration::Session);
    }

    #[test]
    fn expires_format_2() {
        let cookie = Cookie::parse("ID=HelloWorld; Expires=Sun Nov 6 08:49:37 1994").unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert_ne!(cookie.expires(), &Expiration::Session);
    }

    #[test]
    fn expires_format_3() {
        let cookie =
            Cookie::parse("ID=HelloWorld; Expires=Sunday, 06-Nov-94 08:49:37 GMT").unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert_ne!(cookie.expires(), &Expiration::Session);
    }

    #[test]
    fn valid_max_age() {
        let cookie = Cookie::parse("ID=HelloWorld; Max-Age=2000").unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(cookie.max_age().is_some());
        assert_eq!(cookie.max_age().unwrap(), Duration::seconds(2000));
    }

    #[test]
    fn negative_max_age() {
        let cookie = Cookie::parse("ID=HelloWorld; Max-Age=-5000").unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(cookie.max_age().is_some());
        assert_eq!(cookie.max_age().unwrap(), Duration::seconds(0));
    }

    #[test]
    fn no_max_age() {
        let cookie = Cookie::parse("ID=HelloWorld").unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(cookie.max_age().is_none());
    }

    #[test]
    fn domain() {
        let cookie = Cookie::parse("ID=HelloWorld; Domain=google.com").unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(cookie.domain().is_some());
        assert_eq!(
            cookie.domain().as_ref().unwrap(),
            &Host::Domain("google.com".to_string())
        );
    }

    #[test]
    fn no_domain() {
        let cookie = Cookie::parse("ID=HelloWorld").unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(cookie.domain().is_none());
    }

    #[test]
    fn path() {
        let cookie = Cookie::parse("ID=HelloWorld; Path=/hello").unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert_eq!(cookie.path(), "/hello");
    }

    #[test]
    fn no_path() {
        let cookie = Cookie::parse("ID=HelloWorld").unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(cookie.path().is_empty());
    }

    #[test]
    fn is_secure() {
        let cookie = Cookie::parse("ID=HelloWorld; Secure").unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(cookie.secure());
    }

    #[test]
    fn is_not_secure() {
        let cookie = Cookie::parse("ID=HelloWorld").unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(!cookie.secure());
    }

    #[test]
    fn is_http_only() {
        let cookie = Cookie::parse("ID=HelloWorld; HttpOnly").unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(cookie.http_only());
    }

    #[test]
    fn is_not_http_only() {
        let cookie = Cookie::parse("ID=HelloWorld").unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(!cookie.http_only());
    }

    #[test]
    fn google() {
        let cookie1 = Cookie::parse(
            "SOCS=TEST; expires=Tue, 16-Feb-2027 11:39:17 GMT; path=/; domain=.google.com; Secure; SameSite=lax",
        );
        let cookie2 = Cookie::parse(
            "AEC=ABCDEFGHTEST; expires=Thu, 16-Jul-2026 11:39:17 GMT; Path=/; Domain=.google.com; Secure; HttpOnly; SameSite=lax",
        );
        let cookie3 = Cookie::parse(
            "__Secure-ENID=AB.CD=TEST; expires=Wed, 17-Feb-2027 03:57:35 GMT; path=/; domain=.google.com; Secure; HttpOnly; SameSite=lax",
        );

        assert!(cookie1.is_ok());
        let c1 = cookie1.unwrap();

        assert_eq!(c1.name(), "SOCS");
        assert_eq!(c1.value(), "TEST");
        assert!(c1.secure());

        assert!(cookie2.is_ok());
        let c2 = cookie2.unwrap();

        assert_eq!(c2.name(), "AEC");
        assert_eq!(c2.value(), "ABCDEFGHTEST");
        assert!(c2.secure());

        assert!(cookie3.is_ok());
        let c3 = cookie3.unwrap();

        assert_eq!(c3.name(), "__Secure-ENID");
        assert_eq!(c3.value(), "AB.CD=TEST");
        assert!(c3.secure());
    }
}
