use cookies::{CookieDatabase, CookieJar};
use database::Database;
use io::{HttpCache, IndexDatabase};

use crate::errors::CoreError;

pub(crate) struct Databases {
    pub cookie_jar: CookieJar,
    pub http_cache: HttpCache,
}

impl Databases {
    pub fn init() -> Result<Self, CoreError> {
        let cookie_database = CookieDatabase::open().map_err(|e| CoreError::InitializeDatabase(e.to_string()))?;
        let cookie_jar = CookieJar::load(cookie_database);

        let index_database = IndexDatabase::open().map_err(|e| CoreError::InitializeDatabase(e.to_string()))?;
        let http_cache = HttpCache::new(index_database);

        Ok(Self {
            cookie_jar,
            http_cache,
        })
    }
}
