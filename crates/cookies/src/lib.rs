//! Cookies managment library for a browser!

mod cookie;
mod cookie_store;
mod table;

pub use cookie::{Cookie, Expiration};
pub use cookie_store::{CookieJar, StoredCookie};
