use cookies::{CookieDatabase, CookieJar};
use database::Database;

pub(crate) struct Databases {
    pub cookie_jar: CookieJar,
}

impl Databases {
    pub fn init() -> Self {
        let cookie_database = CookieDatabase::open().expect("Couldn't open the Cookie Database");
        let cookie_jar = CookieJar::load(cookie_database);

        Self { cookie_jar }
    }
}
