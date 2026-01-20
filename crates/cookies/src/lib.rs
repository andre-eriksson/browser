//! Cookies managment library for a browser!

mod cookie;
mod cookie_store;
mod table;

pub use cookie::{Cookie, Expiration};
pub use cookie_store::CookieJar;

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
