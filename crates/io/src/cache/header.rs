//! Defines the `CacheHeader` struct which encapsulates metadata for cached HTTP responses,
//! including content type, ETag, last modified time, expiration time, and cache control
//! directives. It also provides methods to determine if a cached entry is still fresh and
//! to generate revalidation headers for conditional requests.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use httpdate::fmt_http_date;
use network::{
    CONTENT_TYPE, ETAG, EXPIRES, HeaderMap, HeaderName, IF_MODIFIED_SINCE, IF_NONE_MATCH,
    LAST_MODIFIED,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const HEADER_VERSION: u16 = 1;

/// Represents the parsed response version of the `Cache-Control` header, containing all relevant
/// directives and their values.
#[derive(Debug, Default)]
pub struct CacheControlResponse {
    pub max_age_seconds: Option<u32>,
    pub s_max_age_seconds: Option<u32>,
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
        let mut response = CacheControlResponse {
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

        for directive in value.split(',').map(|s| s.trim()) {
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
                && let Ok(age) = age_str.parse::<u32>()
            {
                response.max_age_seconds = Some(age);
            } else if let Some(age_str) = directive.strip_prefix("s-max-age=")
                && let Ok(age) = age_str.parse::<u32>()
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

    /// The MIME type of the cached content, extracted from the `Content-Type` header of the HTTP response.
    pub content_type: String,

    // HTTP cache metadata
    /// Optional ETag value from the HTTP response, used for conditional requests during revalidation.
    pub etag: Option<String>,

    /// Optional last modified time of the cached content, represented as a UNIX timestamp in seconds.
    pub last_modified: Option<u64>,

    /// Optional expiration time of the cached content, represented as a UNIX timestamp in seconds.
    /// If `None`, expiration is determined by other cache control directives or heuristics.
    pub expires_at: Option<u64>,

    /// Optional max-age directive from the `Cache-Control` header, representing the maximum age of the cached content in seconds.
    pub max_age_seconds: Option<u32>,

    /// The timestamp when the content was fetched and stored in the cache, represented as a UNIX timestamp in seconds.
    /// This is used to determine freshness and expiration based on max-age and other directives.
    pub fetched_at: u64,

    /// Optional joined headers that are from the Vary header, used to determine which request headers affect the cache key and
    /// to ensure correct cache hits.
    pub vary: Option<String>,

    // Revalidation state
    /// Indicates whether the cached entry must be revalidated with the origin server before being served,
    /// based on the `must-revalidate` directive in the `Cache-Control` header.
    pub must_revalidate: bool,

    /// Indicates whether the cached entry should not be served without revalidation, based on the `no-cache`
    /// directive in the `Cache-Control` header.
    pub no_cache: bool,

    // Data integrity
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
    pub fn new(
        data: &[u8],
        url_hash: [u8; 32],
        vary: &str,
        headers: &HeaderMap,
        cache_control: &CacheControlResponse,
    ) -> Self {
        let content_type = headers
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("text/plain")
            .to_string();

        let etag = headers
            .get(ETAG)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let last_modified = headers
            .get(LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| httpdate::parse_http_date(s).ok())
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs());

        let expires_at = headers
            .get(EXPIRES)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| httpdate::parse_http_date(s).ok())
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs());

        let mut hasher = Sha256::new();
        hasher.update(data);
        let content_hash = hasher.finalize().into();

        let content_size = data.len() as u32;

        Self {
            url_hash,
            content_type,
            etag,
            last_modified,
            expires_at,
            max_age_seconds: cache_control.max_age_seconds,
            fetched_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            vary: if vary.is_empty() {
                None
            } else {
                Some(vary.to_string())
            },
            must_revalidate: cache_control.must_revalidate,
            no_cache: cache_control.no_cache,
            dead: false,
            content_size,
            content_hash,
            header_version: HEADER_VERSION,
        }
    }

    /// Determines if the cached entry is still fresh based on its metadata and the current time.
    /// This method checks the following conditions in order:
    /// 1. If the `no-cache` directive is present, the entry is not fresh and must be revalidated.
    /// 2. If the `expires_at` timestamp is set, the entry is fresh if the current time is before
    ///    the expiration time.
    /// 3. If the `max_age_seconds` directive is set, the entry is fresh if the current time is within
    ///    the max-age window from the `fetched_at` time.
    /// 4. If the `last_modified` timestamp is set, a heuristic freshness lifetime is calculated as
    ///    one-tenth of the time since the last modification, and the entry is fresh if the current
    ///    time is within this heuristic window from the `fetched_at` time.
    /// 5. If none of the above conditions apply, the entry is considered fresh by default.
    pub fn is_fresh(&self) -> bool {
        if self.no_cache {
            return false;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if let Some(expires_at) = self.expires_at {
            return now < expires_at;
        }

        if let Some(max_age) = self.max_age_seconds {
            return now < self.fetched_at + max_age as u64;
        }

        if let Some(last_modified) = self.last_modified {
            let heuristic_ttl = (self.fetched_at.saturating_sub(last_modified)) / 10;
            return now < self.fetched_at + heuristic_ttl;
        }

        true
    }

    /// Generates the necessary headers for revalidating the cached entry with the origin server. This includes:
    /// - `If-None-Match` header with the ETag value if it is present, allowing the server to respond with a
    ///   304 Not Modified if the content has not changed.
    /// - `If-Modified-Since` header with the last modified time if it is present, allowing the server to
    ///   respond with a 304 Not Modified if the content has not changed since that time.
    #[allow(dead_code)]
    pub fn revalidation_headers(&self) -> Vec<(HeaderName, String)> {
        let mut headers = Vec::new();

        if let Some(ref etag) = self.etag {
            headers.push((IF_NONE_MATCH, etag.clone()));
        }

        if let Some(lm) = self.last_modified {
            let last_modified_time = UNIX_EPOCH + Duration::from_secs(lm);
            let last_modified_str = fmt_http_date(last_modified_time);
            headers.push((IF_MODIFIED_SINCE, last_modified_str));
        }

        headers
    }
}
