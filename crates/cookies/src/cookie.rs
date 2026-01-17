use std::fmt::Display;

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
}

impl Cookie {
    pub fn new(
        name: String,
        value: String,
        expires: Expiration,
        max_age: Option<Duration>,
        domain: Option<Host>,
        path: String,
        secure: bool,
        http_only: bool,
    ) -> Self {
        Self {
            name,
            value,
            expires,
            max_age,
            domain,
            path,
            secure,
            http_only,
        }
    }

    pub fn parse(cookie_str: &str) -> Result<Self, String> {
        let parts = cookie_str.split(';');
        let mut cookie = Cookie::default();

        for part in parts {
            let trimmed = part.trim().to_ascii_lowercase();

            if cookie.name.is_empty() {
                let pair = match part.split_once('=') {
                    None => {
                        return Err(format!("Invalid cookie: {}", cookie.name));
                    }
                    Some(pair) => pair,
                };

                cookie.name = String::from(pair.0.trim());
                cookie.value = String::from(pair.1.trim());
            } else if trimmed.starts_with("expires=") {
                let pair: Vec<&str> = part.split('=').collect();
                if pair.len() != 2 {
                    continue;
                }

                let date_parts: Vec<&str> = pair[1].split_ascii_whitespace().collect();

                if date_parts.len() == 6 {
                    // Sun, 06 Nov 1994 08:49:37 GMT
                    let date_format = format_description!("[day]-[month repr:short]-[year]");
                    let full_date = [date_parts[1], date_parts[2], date_parts[3]].join("-");

                    let date = match Date::parse(full_date.as_str(), date_format) {
                        Err(_) => continue,
                        Ok(date) => date,
                    };

                    let time_format = format_description!("[hour]:[minute]:[second]");
                    let time = match Time::parse(date_parts[4], time_format) {
                        Err(_) => continue,
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
                        Err(e) => {
                            eprintln!("{e}");
                            continue;
                        }
                        Ok(date) => date,
                    };

                    let time_format = format_description!("[hour]:[minute]:[second]");
                    let time = match Time::parse(date_parts[3], time_format) {
                        Err(e) => {
                            eprintln!("{e}");
                            continue;
                        }
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
                        Err(e) => {
                            dbg!(correct_date.trim());
                            eprintln!("Date: {e}");
                            continue;
                        }
                        Ok(date) => date,
                    };

                    let time_format = format_description!("[hour]:[minute]:[second]");
                    let time = match Time::parse(date_parts[2], time_format) {
                        Err(e) => {
                            eprintln!("Time: {e}");
                            continue;
                        }
                        Ok(parsed) => parsed,
                    };

                    cookie.expires =
                        Expiration::Date(OffsetDateTime::new_in_offset(date, time, UtcOffset::UTC))
                }
            } else if trimmed.starts_with("max-age=") {
                let pair: Vec<&str> = part.split('=').collect();
                if pair.len() != 2 {
                    continue;
                }

                let value = if pair[1].trim().starts_with('-') {
                    // TODO: Something?
                    "0"
                } else {
                    pair[1].trim()
                };

                let val = match value.parse::<i64>() {
                    Err(_) => continue,
                    Ok(val) => val,
                };

                let duration = Duration::seconds(val);

                cookie.max_age = Some(duration);
            } else if trimmed.starts_with("domain=") {
                let pair: Vec<&str> = part.split('=').collect();
                if pair.len() != 2 {
                    continue;
                }

                let mut value = pair[1].trim();

                if value.starts_with('.') {
                    value = &value[1..];
                }

                let domain = match Host::parse(value) {
                    Err(_) => continue,
                    Ok(host) => host,
                };

                cookie.domain = Some(domain);
            } else if trimmed.starts_with("path=") {
                let pair: Vec<&str> = part.split('=').collect();
                if pair.len() != 2 {
                    continue;
                }
                let value = pair[1].trim();

                if value.starts_with('/') || !value.is_empty() {
                    cookie.path = String::from(pair[1].trim());
                } // TODO: Handle "default-path"
            } else if trimmed == "secure" {
                cookie.secure = true;
            } else if trimmed == "httponly" {
                cookie.http_only = true;
            }
        }

        Ok(cookie)
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
}

impl Display for Cookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.name(), self.value())
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
            "AEC=ABCDEFGHTEST; expires=Thu, 16-Jul-2026 11:39:17 GMT; path=/; domain=.google.com; Secure; HttpOnly; SameSite=lax",
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
