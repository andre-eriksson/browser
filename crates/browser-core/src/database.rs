use cookies::{CookieDatabase, CookieJar};
use database::Database;
use io::{HttpCache, IndexDatabase};
use network::response::Response;

pub(crate) struct Databases {
    pub cookie_jar: CookieJar,
    pub http_cache: HttpCache<String, Response>,
}

impl Databases {
    pub fn init() -> Self {
        let cookie_database = CookieDatabase::open().expect("Couldn't open the Cookie Database");
        let cookie_jar = CookieJar::load(cookie_database);

        let index_database = IndexDatabase::open().expect("Couldn't open the Index Database");
        let http_cache = HttpCache::new(index_database);

        Self {
            cookie_jar,
            http_cache,
        }
    }
}
