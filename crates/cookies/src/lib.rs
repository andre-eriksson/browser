//! Cookies managment library for a browser!

mod cookie;
mod cookie_store;
pub mod errors;
mod table;

pub use cookie::{Cookie, Expiration};
pub use cookie_store::CookieJar;

#[cfg(test)]
mod tests {
    use time::Duration;
    use url::{Host, Url};

    use crate::{cookie::Cookie, cookie::Expiration};

    fn localhost() -> Url {
        Url::parse("http://localhost").unwrap()
    }

    #[test]
    fn test_expires_format_1() {
        let cookie = Cookie::parse(
            "ID=HelloWorld; Expires=Sun, 06 Nov 1994 08:49:37 GMT",
            &localhost(),
        )
        .unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert_ne!(cookie.expires(), &Expiration::Session);
    }

    #[test]
    fn test_expires_format_2() {
        let cookie = Cookie::parse(
            "ID=HelloWorld; Expires=Sun Nov 6 08:49:37 1994",
            &localhost(),
        )
        .unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert_ne!(cookie.expires(), &Expiration::Session);
    }

    #[test]
    fn test_expires_format_3() {
        let cookie = Cookie::parse(
            "ID=HelloWorld; Expires=Sunday, 06-Nov-94 08:49:37 GMT",
            &localhost(),
        )
        .unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert_ne!(cookie.expires(), &Expiration::Session);
    }

    #[test]
    fn test_valid_max_age() {
        let cookie = Cookie::parse("ID=HelloWorld; Max-Age=2000", &localhost()).unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(cookie.max_age().is_some());
        assert_eq!(cookie.max_age().unwrap(), Duration::seconds(2000));
    }

    #[test]
    fn test_negative_max_age() {
        let cookie = Cookie::parse("ID=HelloWorld; Max-Age=-5000", &localhost()).unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(cookie.max_age().is_some());
        assert_eq!(cookie.max_age().unwrap(), Duration::seconds(0));
    }

    #[test]
    fn test_no_max_age() {
        let cookie = Cookie::parse("ID=HelloWorld", &localhost()).unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(cookie.max_age().is_none());
    }

    #[test]
    fn test_domain() {
        let cookie = Cookie::parse("ID=HelloWorld; Domain=google.com", &localhost()).unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(cookie.domain().is_some());
        assert_eq!(
            *cookie.domain().as_ref().unwrap(),
            Host::Domain("google.com".to_string()).into()
        );
    }

    #[test]
    fn test_no_domain() {
        let cookie = Cookie::parse("ID=HelloWorld", &localhost()).unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(cookie.domain().is_none());
    }

    #[test]
    fn test_path() {
        let cookie = Cookie::parse("ID=HelloWorld; Path=/hello", &localhost()).unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert_eq!(cookie.path(), "/hello");
    }

    #[test]
    fn test_no_path() {
        let cookie = Cookie::parse("ID=HelloWorld", &localhost()).unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(cookie.path().is_empty());
    }

    #[test]
    fn test_is_secure() {
        let cookie = Cookie::parse("ID=HelloWorld; Secure", &localhost()).unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(cookie.secure());
    }

    #[test]
    fn test_is_not_secure() {
        let cookie = Cookie::parse("ID=HelloWorld", &localhost()).unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(!cookie.secure());
    }

    #[test]
    fn test_is_http_only() {
        let cookie = Cookie::parse("ID=HelloWorld; HttpOnly", &localhost()).unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(cookie.http_only());
    }

    #[test]
    fn test_is_not_http_only() {
        let cookie = Cookie::parse("ID=HelloWorld", &localhost()).unwrap();

        assert_eq!(cookie.name(), "ID");
        assert_eq!(cookie.value(), "HelloWorld");
        assert!(!cookie.http_only());
    }

    #[test]
    fn test_host_http_cookie_prefix() {
        let http_host_cookie_valid = Cookie::parse(
            "__Host-Http-ID=HelloWorld; Path=/; Secure; HttpOnly",
            &localhost(),
        )
        .unwrap();
        assert!(Cookie::validate_cookie_prefix(&http_host_cookie_valid).is_ok());

        let http_host_cookie_invalid_not_secure = Cookie::builder()
            .name(String::from("__Host-Http-ID=HelloWorld"))
            .value(String::from("HelloWorld"))
            .path(String::from("/"))
            .http_only(true)
            .build_unchecked();

        assert!(Cookie::validate_cookie_prefix(&http_host_cookie_invalid_not_secure).is_err());

        let http_host_cookie_invalid_http_only = Cookie::builder()
            .name(String::from("__Host-Http-ID=HelloWorld"))
            .value(String::from("HelloWorld"))
            .path(String::from("/"))
            .secure(true)
            .build_unchecked();

        assert!(Cookie::validate_cookie_prefix(&http_host_cookie_invalid_http_only).is_err());

        let http_host_cookie_invalid_domain = Cookie::builder()
            .name(String::from("__Host-Http-ID=HelloWorld"))
            .value(String::from("HelloWorld"))
            .domain(Host::Domain("malicious.com".to_string()))
            .secure(true)
            .http_only(true)
            .build_unchecked();

        assert!(Cookie::validate_cookie_prefix(&http_host_cookie_invalid_domain).is_err());

        let http_host_cookie_invalid_path = Cookie::builder()
            .name(String::from("__Host-Http-ID=HelloWorld"))
            .value(String::from("HelloWorld"))
            .secure(true)
            .http_only(true)
            .build_unchecked();

        assert!(Cookie::validate_cookie_prefix(&http_host_cookie_invalid_path).is_err());
    }

    #[test]
    fn test_host_cookie_prefix() {
        let host_cookie_valid =
            Cookie::parse("__Host-ID=HelloWorld; Path=/; Secure", &localhost()).unwrap();
        assert!(Cookie::validate_cookie_prefix(&host_cookie_valid).is_ok());

        let host_cookie_invalid_not_secure = Cookie::builder()
            .name(String::from("__Host-ID=HelloWorld"))
            .value(String::from("HelloWorld"))
            .path(String::from("/"))
            .build_unchecked();

        assert!(Cookie::validate_cookie_prefix(&host_cookie_invalid_not_secure).is_err());

        let host_cookie_invalid_domain = Cookie::builder()
            .name(String::from("__Host-ID=HelloWorld"))
            .value(String::from("HelloWorld"))
            .domain(Host::Domain("malicious.com".to_string()))
            .secure(true)
            .build_unchecked();

        assert!(Cookie::validate_cookie_prefix(&host_cookie_invalid_domain).is_err());

        let host_cookie_invalid_path = Cookie::builder()
            .name(String::from("__Host-ID=HelloWorld"))
            .value(String::from("HelloWorld"))
            .secure(true)
            .build_unchecked();

        assert!(Cookie::validate_cookie_prefix(&host_cookie_invalid_path).is_err());
    }

    #[test]
    fn test_http_cookie_prefix() {
        let http_cookie_valid =
            Cookie::parse("__Http-ID=HelloWorld; HttpOnly; Secure", &localhost()).unwrap();
        assert!(Cookie::validate_cookie_prefix(&http_cookie_valid).is_ok());

        let http_cookie_invalid_no_httponly = Cookie::builder()
            .name(String::from("__Http-ID=HelloWorld"))
            .value(String::from("HelloWorld"))
            .secure(true)
            .build_unchecked();

        assert!(Cookie::validate_cookie_prefix(&http_cookie_invalid_no_httponly).is_err());

        let http_cookie_invalid_no_secure = Cookie::builder()
            .name(String::from("__Http-ID=HelloWorld"))
            .value(String::from("HelloWorld"))
            .http_only(true)
            .build_unchecked();

        assert!(Cookie::validate_cookie_prefix(&http_cookie_invalid_no_secure).is_err());
    }

    #[test]
    fn test_secure_cookie_prefix() {
        let secure_cookie = Cookie::parse("__Secure-ID=HelloWorld; Secure", &localhost()).unwrap();
        assert!(Cookie::validate_cookie_prefix(&secure_cookie).is_ok());

        let no_secure_cookie = Cookie::builder()
            .name(String::from("__Secure-ID=HelloWorld"))
            .value(String::from("HelloWorld"))
            .build_unchecked();

        assert!(Cookie::validate_cookie_prefix(&no_secure_cookie).is_err());
    }

    #[test]
    fn test_google_cookie_format() {
        let cookie1 = Cookie::parse(
            "SOCS=TEST; expires=Tue, 16-Feb-2027 11:39:17 GMT; path=/; domain=.google.com; Secure; SameSite=lax",
            &localhost(),
        );
        let cookie2 = Cookie::parse(
            "AEC=ABCDEFGHTEST; expires=Thu, 16-Jul-2026 11:39:17 GMT; Path=/; Domain=.google.com; Secure; HttpOnly; SameSite=lax",
            &localhost(),
        );
        let cookie3 = Cookie::parse(
            "__Secure-ENID=AB.CD=TEST; expires=Wed, 17-Feb-2027 03:57:35 GMT; path=/; domain=.google.com; Secure; HttpOnly; SameSite=lax",
            &localhost(),
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
