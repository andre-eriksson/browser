//! In-memory cache implementation for resources, with support for disk persistence and Vary header handling.

use network::{CACHE_CONTROL, HeaderMap, HeaderName, response::Response};
use postcard::to_stdvec;
use sha2::{Digest, Sha256};
use std::{fmt::Debug, str::FromStr};
use tracing::debug;

use crate::cache::{
    disk::DiskCache,
    errors::{CacheError, CacheRead},
    header::{CacheControlResponse, CacheHeader, HEADER_VERSION},
    index::IndexDatabase,
};

/// The current state of a cached resource in memory.
#[derive(Debug, Clone)]
pub enum CacheEntry {
    /// The resource has been successfully loaded.
    Loaded(Box<CacheRead>),
    /// The resource failed to load.
    Failed,
}

impl From<CacheRead> for CacheEntry {
    fn from(value: CacheRead) -> Self {
        Self::Loaded(Box::new(value))
    }
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
    pub fn get(&self, key: &str, headers: &HeaderMap) -> Result<CacheEntry, CacheError> {
        let sha = Self::hash_url(key);

        let Some(entry) = self.inner.get(sha)? else {
            return Ok(CacheEntry::Failed);
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
            // TODO: Reason?
            return Ok(CacheEntry::Failed);
        }

        let mut hasher = Sha256::new();
        hasher.update(&data);
        let content_hash: [u8; 32] = hasher.finalize().into();

        if cache_header.content_hash != content_hash {
            return Ok(CacheEntry::Failed);
        }

        let deserialized: Response = postcard::from_bytes(&data).map_err(CacheError::Serialization)?;

        let vary_header_names = index
            .vary
            .iter()
            .filter_map(|v| HeaderName::from_str(v).ok())
            .collect::<Vec<HeaderName>>();

        for header_name in vary_header_names {
            let cached = deserialized.headers.get(&header_name);
            let current = headers.get(&header_name);

            if cached != current {
                return Ok(CacheEntry::Failed);
            }
        }

        if !index.is_fresh() {
            return Ok(CacheRead::RequiresRevalidation {
                stale_data: deserialized,
                revalidation_headers: index.revalidation_headers(),
            }
            .into());
        }

        Ok(CacheRead::Hit(deserialized).into())
    }

    /// Stores a successfully loaded value in the cache for a given key.
    ///
    /// # Errors
    /// * If there is already an entry for the key in the cache.
    /// * If there is an error writing to disk or serializing the value.
    pub fn store(&self, key: String, value: Response, headers: &HeaderMap) -> Result<(), CacheError> {
        if let Err(error) = self.store_on_disk(&key, &value, headers) {
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
    fn store_on_disk(&self, key: &str, value: &Response, headers: &HeaderMap) -> Result<(), CacheError> {
        let sha = Self::hash_url(key);

        let serialized = to_stdvec(value).map_err(CacheError::Serialization)?;

        let cache_control = CacheControlResponse::from(
            headers
                .get(CACHE_CONTROL)
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default(),
        );

        if cache_control.no_store {
            return Err(CacheError::Write(String::from("\"cache-control: no-store\" prevents caching")));
        }

        let cache_headers = CacheHeader::new(serialized.as_slice(), sha);

        self.inner
            .put(sha, serialized.as_slice(), headers, &cache_control, cache_headers)
    }

    pub fn revalidate(&self, key: &str, headers: &HeaderMap) -> Result<(), CacheError> {
        let sha = Self::hash_url(key);

        self.inner.revalidate(sha, headers)
    }

    /// Evicts a cache entry for a given key, removing it from both memory and disk.
    ///
    /// # Errors
    /// * If there is an error removing the entry from disk.
    pub fn evict(&mut self, key: &str, _headers: &HeaderMap) -> Result<bool, CacheError> {
        let sha = Self::hash_url(key);

        let removed_disk = self.inner.delete(sha)?;
        Ok(removed_disk)
    }

    /// Hashes a URL to produce a unique key for caching.
    fn hash_url(url: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        hasher.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use database::Database;
    use network::VARY;
    use serial_test::serial;

    use super::*;

    #[test]
    #[serial]
    fn test_store_and_get() {
        let mut headers = HeaderMap::new();
        headers.insert(VARY, "Accept-Encoding".parse().unwrap());
        headers.insert("Accept-Encoding", "gzip".parse().unwrap());
        let key = "https://example.com/resource".to_string();
        let value = Response::from("cached data".as_bytes().to_vec());

        let database = IndexDatabase::open().expect("Couldn't open database");
        let cache = HttpCache::new(database);

        let result = cache.store_on_disk(&key, &value, &headers);
        assert!(result.is_ok());

        let mut headers = HeaderMap::new();
        headers.insert(VARY, "Accept-Encoding".parse().unwrap());
        headers.insert("Accept-Encoding", "gzip".parse().unwrap());

        let retrieved = cache.get(&key, &headers);

        assert!(retrieved.is_ok());
        //assert_eq!(value, retrieved.unwrap());
    }

    // #[test]
    // #[serial]
    // fn test_get_with_vary_from_disk() {
    //     let mut headers = HeaderMap::new();
    //     headers.insert(VARY, "Accept-Encoding".parse().unwrap());
    //     headers.insert("Accept-Encoding", "br".parse().unwrap());

    //     let database = IndexDatabase::open().expect("Couldn't open database");
    //     let cache = HttpCache::new(database);

    //     let key = "https://example.com/vary-test".to_string();
    //     let value = "vary cached data".to_string();

    //     let vary = HttpCache::<String, String>::resolve_vary(&headers).unwrap();
    //     assert!(!vary.is_empty());

    //     cache.store_on_disk(&key, &value, &headers).unwrap();

    //     cache.entries.write().unwrap().clear();

    //     let result = cache.get_with_vary(&key, &vary).unwrap();
    //     match result {
    //         CacheEntry::Loaded(read) => match (*read).clone() {
    //             CacheRead::Hit(v) => assert_eq!(v, value),
    //             CacheRead::Miss => panic!("Expected Hit, got Miss"),
    //         },
    //         other => panic!("Expected Loaded, got {:?}", other),
    //     }
    // }

    // #[test]
    // #[serial]
    // fn test_get_with_vary_wrong_vary_misses() {
    //     let mut headers = HeaderMap::new();
    //     headers.insert(VARY, "Accept-Encoding".parse().unwrap());
    //     headers.insert("Accept-Encoding", "gzip".parse().unwrap());

    //     let database = IndexDatabase::open().expect("Couldn't open database");
    //     let cache = HttpCache::new(database);

    //     let key = "https://example.com/vary-miss-test".to_string();
    //     let value = "gzip data".to_string();

    //     cache.store_on_disk(&key, &value, &headers).unwrap();

    //     cache.entries.write().unwrap().clear();

    //     let wrong_vary = "accept-encoding:br".to_string();
    //     let result = cache.get_with_vary(&key, &wrong_vary).unwrap();
    //     match result {
    //         CacheEntry::Pending => {}
    //         other => panic!("Expected Pending (miss), got {:?}", other),
    //     }
    // }
}
