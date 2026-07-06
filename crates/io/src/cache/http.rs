//! In-memory cache implementation for resources, with support for disk persistence and Vary header handling.

use network::{CACHE_CONTROL, HeaderMap, VARY, response::Response};
use postcard::to_stdvec;
use sha2::{Digest, Sha256};
use std::fmt::Debug;
use storage::Directory;
use tracing::debug;

use crate::cache::{
    disk::DiskCache,
    errors::CacheError,
    header::{CacheControlResponse, CacheHeader, HEADER_VERSION},
    index::IndexDatabase,
};

/// The current state of a cached resource in memory.
#[derive(Debug, Clone)]
pub enum CacheEntry {
    Hit(Response),
    RequiresRevalidation {
        /// The stale data to use if the server responds with 304 Not Modified
        stale_data: Response,
        /// The headers to attach to the outbound request (e.g., If-None-Match)
        revalidation_headers: HeaderMap,
    },
    Miss,
}

/// A thread-safe cache for resources, keyed by a generic key type `K`.
#[derive(Debug, Clone)]
pub struct HttpCache {
    inner: DiskCache,
}

impl HttpCache {
    /// Creates a new empty cache.
    #[must_use]
    pub fn new(database: IndexDatabase) -> Self {
        Self {
            inner: DiskCache::new(database),
        }
    }

    /// Gets the cache entry for a given key, if it exists.
    ///
    /// # Errors
    /// * If the cache lock is poisoned.
    /// * If there is an error reading from disk or deserializing the cached value.
    pub fn get(&self, dirs: &Directory, key: &str, request_headers: &HeaderMap) -> Result<CacheEntry, CacheError> {
        let sha = Self::hash_url(key);

        let Some(entry) = self.inner.get(dirs, sha, request_headers)? else {
            return Ok(CacheEntry::Miss);
        };

        let content_size = entry.content_size;
        let cache_header = entry.header;
        let data = entry.data;
        let index = entry.index;

        if cache_header.url_hash != sha
            || cache_header.content_size != u32::try_from(content_size).unwrap_or(u32::MAX)
            || cache_header.header_version != HEADER_VERSION
            || cache_header.dead
        {
            return Ok(CacheEntry::Miss);
        }

        let mut hasher = Sha256::new();
        hasher.update(&data);
        let content_hash: [u8; 32] = hasher.finalize().into();

        if cache_header.content_hash != content_hash {
            return Ok(CacheEntry::Miss);
        }

        let deserialized: Response = postcard::from_bytes(&data).map_err(CacheError::Serialization)?;

        if !index.is_fresh() {
            return Ok(CacheEntry::RequiresRevalidation {
                stale_data: deserialized,
                revalidation_headers: index.revalidation_headers(),
            });
        }

        Ok(CacheEntry::Hit(deserialized))
    }

    /// Stores a successfully loaded value in the cache for a given key.
    ///
    /// # Errors
    /// * If there is already an entry for the key in the cache.
    /// * If there is an error writing to disk or serializing the value.
    pub fn store(
        &self,
        dirs: &Directory,
        key: String,
        response: Response,
        request_headers: &HeaderMap,
    ) -> Result<(), CacheError> {
        if let Err(error) = self.store_on_disk(dirs, &key, &response, request_headers) {
            debug!(%error, "failed to store on disk");
            return Err(error);
        }

        Ok(())
    }

    /// Stores a value on disk for a given key and headers, handling Vary header resolution and cache control directives.
    ///
    /// # Errors
    /// * If the `Vary` header is invalid or prevents caching.
    /// * If the `Cache-Control` header prevents caching.
    /// * If there is an error writing to disk or serializing the value.
    fn store_on_disk(
        &self,
        dirs: &Directory,
        key: &str,
        response: &Response,
        request_headers: &HeaderMap,
    ) -> Result<(), CacheError> {
        let sha = Self::hash_url(key);

        let serialized = to_stdvec(response).map_err(CacheError::Serialization)?;

        let cache_control = CacheControlResponse::from(
            response
                .headers
                .get(CACHE_CONTROL)
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default(),
        );

        if cache_control.no_store {
            return Ok(());
        }

        let cache_headers = CacheHeader::new(serialized.as_slice(), sha);

        self.inner.put(
            dirs,
            sha,
            serialized.as_slice(),
            &response.headers,
            request_headers,
            &cache_control,
            cache_headers,
        )
    }

    pub fn revalidate(&self, key: &str, request_headers: &HeaderMap) -> Result<(), CacheError> {
        let sha = Self::hash_url(key);

        self.inner.revalidate(sha, request_headers)
    }

    /// Evicts a cache entry for a given key, removing it from both memory and disk.
    ///
    /// # Errors
    /// * If there is an error removing the entry from disk.
    pub fn evict(&mut self, dirs: &Directory, key: &str, request_headers: &HeaderMap) -> Result<bool, CacheError> {
        let sha = Self::hash_url(key);

        let removed_disk = self.inner.delete(dirs, sha, request_headers)?;
        Ok(removed_disk)
    }

    /// Hashes a URL to produce a unique key for caching.
    fn hash_url(url: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        hasher.finalize().into()
    }

    pub(crate) fn hash_vary(vary: &[String], request_headers: &HeaderMap) -> [u8; 32] {
        let mut hasher = Sha256::new();

        for header_name in vary {
            if let Some(value) = request_headers.get(header_name) {
                hasher.update(header_name.as_bytes());
                hasher.update(value.as_bytes());
            }
        }

        hasher.finalize().into()
    }

    /// Deserializes a Vary header value into a vector of header names.
    pub(crate) fn deserialize_vary(vary: Option<String>) -> Vec<String> {
        vary.unwrap_or_default()
            .split(',')
            .map(String::from)
            .collect::<Vec<String>>()
    }

    /// Serializes a vector of header names into a Vary header value.
    pub(crate) fn serialize_vary(vary: &[String]) -> String {
        vary.join(",")
    }

    /// Extracts the Vary header from the given headers and returns a vector of header names.
    pub(crate) fn extract_vary(headers: &HeaderMap) -> Vec<String> {
        headers
            .get(VARY)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').map(|h| h.trim().to_string()).collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use database::Database;
    use http_serde::http::StatusCode;
    use network::VARY;
    use serial_test::serial;
    use storage::Directory;

    use super::*;

    #[test]
    fn test_hash_url() {
        let url = "https://example.com/resource";
        let hash1 = HttpCache::hash_url(url);
        let hash2 = HttpCache::hash_url(url);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_vary() {
        let vary_headers = vec!["Accept-Encoding".to_string(), "User-Agent".to_string()];
        let mut request_headers = HeaderMap::new();
        request_headers.insert("Accept-Encoding", "gzip".parse().unwrap());
        request_headers.insert("User-Agent", "Mozilla/5.0".parse().unwrap());

        let hash1 = HttpCache::hash_vary(&vary_headers, &request_headers);
        let hash2 = HttpCache::hash_vary(&vary_headers, &request_headers);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_serialize_deserialize_vary() {
        let vary_headers = vec!["Accept-Encoding".to_string(), "User-Agent".to_string()];
        let serialized = HttpCache::serialize_vary(&vary_headers);
        assert_eq!(serialized, "Accept-Encoding,User-Agent");

        let deserialized = HttpCache::deserialize_vary(Some(serialized));
        assert_eq!(deserialized, vary_headers);
    }

    #[test]
    fn test_extract_vary() {
        let mut headers = HeaderMap::new();
        headers.insert(VARY, "Accept-Encoding, User-Agent".parse().unwrap());

        let vary_headers = HttpCache::extract_vary(&headers);
        assert_eq!(vary_headers, vec!["Accept-Encoding".to_string(), "User-Agent".to_string()]);
    }

    #[test]
    #[serial]
    fn test_store_and_get() {
        let mut request_header = HeaderMap::new();
        request_header.insert("Accept-Encoding", "gzip".parse().unwrap());

        let mut response_headers = HeaderMap::new();
        response_headers.insert(VARY, "Accept-Encoding".parse().unwrap());
        response_headers.insert("Accept-Encoding", "gzip".parse().unwrap());
        let key = "https://example.com/resource".to_string();
        let response = Response::new(StatusCode::OK, response_headers, Some("cached_data".as_bytes().to_vec()));

        let database = IndexDatabase::open(Directory::try_new().unwrap()).expect("Couldn't open database");
        let cache = HttpCache::new(database);
        let dirs = Directory::try_new().unwrap();

        let result = cache.store(&dirs, key.clone(), response, &request_header);
        assert!(result.is_ok());

        let retrieved = cache.get(&dirs, &key, &request_header);

        assert!(retrieved.is_ok());
        assert!(matches!(retrieved.unwrap(), CacheEntry::Hit(_)));
    }

    #[test]
    #[serial]
    fn test_vary_header_handling() {
        let mut request_header = HeaderMap::new();
        request_header.insert("Accept-Encoding", "gzip".parse().unwrap());

        let mut response_headers = HeaderMap::new();
        response_headers.insert(VARY, "Accept-Encoding".parse().unwrap());
        response_headers.insert("Accept-Encoding", "gzip".parse().unwrap());
        let key = "https://example.com/resource".to_string();
        let response = Response::new(StatusCode::OK, response_headers, Some("cached_data".as_bytes().to_vec()));

        let database = IndexDatabase::open(Directory::try_new().unwrap()).expect("Couldn't open database");
        let cache = HttpCache::new(database);
        let dirs = Directory::try_new().unwrap();

        let result = cache.store(&dirs, key.clone(), response, &request_header);
        assert!(result.is_ok());

        // Attempt to retrieve with a different Accept-Encoding value
        let mut different_request_header = HeaderMap::new();
        different_request_header.insert("Accept-Encoding", "deflate".parse().unwrap());

        let retrieved = cache.get(&dirs, &key, &different_request_header);

        assert!(retrieved.is_ok());
        assert!(matches!(retrieved.unwrap(), CacheEntry::Miss));
    }
}
