use std::time::{Duration, SystemTime, UNIX_EPOCH};

use httpdate::fmt_http_date;
use network::{
    CONTENT_TYPE, ETAG, EXPIRES, HeaderMap, HeaderName, IF_MODIFIED_SINCE, IF_NONE_MATCH,
    LAST_MODIFIED,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const HEADER_VERSION: u16 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheHeader {
    // Identity
    pub url_hash: [u8; 32],
    pub content_type: String,

    // HTTP cache metadata
    pub etag: Option<String>,
    pub last_modified: Option<u64>,
    pub expires_at: Option<u64>,
    pub max_age_seconds: Option<u32>,
    pub fetched_at: u64,
    pub vary: Option<String>,

    // Revalidation state
    pub must_revalidate: bool,
    pub no_cache: bool,

    // Data integrity
    pub content_size: u32,
    pub content_hash: [u8; 32],
    pub header_version: u16,
}

impl CacheHeader {
    pub fn new(data: &[u8], url_hash: [u8; 32], vary: &str, headers: &HeaderMap) -> Self {
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

        let (max_age_seconds, no_cache, must_revalidate) = Self::parse_cache_control(headers);

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
            max_age_seconds,
            fetched_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            vary: if vary.is_empty() {
                None
            } else {
                Some(vary.to_string())
            },
            must_revalidate,
            no_cache,
            content_size,
            content_hash,
            header_version: HEADER_VERSION,
        }
    }

    #[allow(dead_code)]
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
            let heuristic_ttl = (self.fetched_at - last_modified) / 10;
            return now < self.fetched_at + heuristic_ttl;
        }

        false
    }

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

    fn parse_cache_control(headers: &HeaderMap) -> (Option<u32>, bool, bool) {
        let mut max_age = None;
        let mut no_cache = false;
        let mut must_revalidate = false;

        if let Some(cache_control) = headers.get("Cache-Control").and_then(|v| v.to_str().ok()) {
            for directive in cache_control.split(',').map(|s| s.trim()) {
                if directive.eq_ignore_ascii_case("no-cache") {
                    no_cache = true;
                } else if directive.eq_ignore_ascii_case("must-revalidate") {
                    must_revalidate = true;
                } else if let Some(age_str) = directive.strip_prefix("max-age=")
                    && let Ok(age) = age_str.parse::<u32>()
                {
                    max_age = Some(age);
                }
            }
        }

        (max_age, no_cache, must_revalidate)
    }
}
