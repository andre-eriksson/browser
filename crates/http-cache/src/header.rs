use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const HEADER_VERSION: u16 = 1;

/// Represents the parsed response version of the `Cache-Control` header, containing all relevant
/// directives and their values.
#[derive(Debug, Default)]
pub struct CacheControlResponse {
    pub max_age_seconds: Option<u64>,
    pub s_max_age_seconds: Option<u64>,
    pub no_cache: bool,
    pub must_revalidate: bool,
    pub proxy_revalidate: bool,
    pub no_store: bool,
    pub private: bool,
    pub public: bool,
    pub must_understand: bool,
    pub no_transform: bool,
    pub immutable: bool,
}

impl From<&str> for CacheControlResponse {
    fn from(value: &str) -> Self {
        let mut response = Self {
            max_age_seconds: None,
            s_max_age_seconds: None,
            no_cache: false,
            must_revalidate: false,
            proxy_revalidate: false,
            no_store: false,
            private: false,
            public: false,
            must_understand: false,
            no_transform: false,
            immutable: false,
        };

        for directive in value.split(',').map(str::trim) {
            if directive.eq_ignore_ascii_case("no-cache") {
                response.no_cache = true;
            } else if directive.eq_ignore_ascii_case("must-revalidate") {
                response.must_revalidate = true;
            } else if directive.eq_ignore_ascii_case("proxy-revalidate") {
                response.proxy_revalidate = true;
            } else if directive.eq_ignore_ascii_case("no-store") {
                response.no_store = true;
            } else if directive.eq_ignore_ascii_case("private") {
                response.private = true;
            } else if directive.eq_ignore_ascii_case("public") {
                response.public = true;
            } else if directive.eq_ignore_ascii_case("must-understand") {
                response.must_understand = true;
            } else if directive.eq_ignore_ascii_case("no-transform") {
                response.no_transform = true;
            } else if directive.eq_ignore_ascii_case("immutable") {
                response.immutable = true;
            } else if let Some(age_str) = directive.strip_prefix("max-age=")
                && let Ok(age) = age_str.parse::<u64>()
            {
                response.max_age_seconds = Some(age);
            } else if let Some(age_str) = directive.strip_prefix("s-max-age=")
                && let Ok(age) = age_str.parse::<u64>()
            {
                response.s_max_age_seconds = Some(age);
            }
        }

        response
    }
}

/// Represents the metadata header for a cached HTTP response, containing all necessary information to
/// manage cache freshness, revalidation, and data integrity.
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheHeader {
    // Identity
    /// SHA-256 hash of the URL, used as the key for cache lookup and to ensure data integrity.
    pub url_hash: [u8; 32],

    // Integrity
    /// Indicates whether the cached entry is considered "dead" or invalid, which can occur if the
    /// content has been modified on the origin server or if the cache entry has been corrupted.
    /// Dead entries should not be served and should be removed from the cache during cleanup.
    pub dead: bool,

    /// The size of the cached content in bytes, used for managing cache storage and ensuring data integrity during retrieval.
    pub content_size: u32,

    /// SHA-256 hash of the cached content, used to verify data integrity when retrieving from the cache and to detect corruption.
    pub content_hash: [u8; 32],

    /// The version of the cache header format, used to ensure compatibility when reading and writing cache entries.
    pub header_version: u16,
}

impl CacheHeader {
    pub fn new(data: &[u8], url_hash: [u8; 32]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let content_hash = hasher.finalize().into();

        let content_size = u32::try_from(data.len()).unwrap_or(u32::MAX);

        Self {
            url_hash,
            dead: false,
            content_size,
            content_hash,
            header_version: HEADER_VERSION,
        }
    }
}
