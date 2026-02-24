use network::{CACHE_CONTROL, HeaderMap, HeaderName, VARY};
use postcard::to_stdvec;
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, RwLock},
};

use crate::cache::{
    block::BlockFile,
    errors::CacheError,
    header::{CacheControlResponse, CacheHeader, HEADER_VERSION},
    index::{IndexFile, PointerType},
    large::LargeFile,
};

pub mod block;
pub mod errors;
pub mod header;
pub mod index;
pub mod large;

const MAX_BLOCK_SIZE: u64 = 20_000_000; // 20 MB

/// The current state of a cached resource in memory.
#[derive(Debug, Clone)]
pub enum CacheEntry<T> {
    /// The resource has been requested but not yet loaded.
    Pending,
    /// The resource has been successfully loaded.
    Loaded(Arc<T>),
    /// The resource failed to load.
    Failed,
}

/// A thread-safe cache for resources, keyed by a generic key type `K`.
#[derive(Debug, Clone, Default)]
pub struct Cache<K, V> {
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
}

impl<K, V> Cache<K, V>
where
    K: AsRef<str> + Eq + Hash + Clone + Serialize + DeserializeOwned,
    V: Clone + Serialize + DeserializeOwned,
{
    /// Creates a new empty cache.
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Checks if a key is already in the cache (in any state).
    pub fn contains(&self, key: &K) -> bool {
        self.entries
            .read()
            .map(|entries| entries.contains_key(key))
            .unwrap_or(false)
    }

    /// Gets the cache entry for a given key, if it exists.
    pub fn get(&self, key: &K) -> Option<CacheEntry<V>> {
        self.entries
            .read()
            .ok()
            .and_then(|entries| entries.get(key).cloned())
    }

    pub fn get_idx(&self, key: &K, headers: &HeaderMap) -> Option<V> {
        let vary = Self::resolve_vary(headers).ok()?;
        let sha = Self::hash_url(key.as_ref(), &vary);

        let idx_file = IndexFile::load()?;

        let pointer = idx_file.entries.get(&sha)?;

        let (header, value, content_size) = match pointer {
            PointerType::Large => LargeFile::read::<V>(sha),
            PointerType::Block(ptr) => BlockFile::read(ptr),
        }?;

        if header.url_hash != sha
            || header.content_size != content_size as u32
            || header.header_version != HEADER_VERSION
            || !header.is_fresh()
        {
            return None;
        }

        Some(value)
    }

    /// Inserts or updates a cache entry for a given key.
    pub fn insert(&self, key: K, entry: CacheEntry<V>) {
        if let Ok(mut entries) = self.entries.write() {
            entries.insert(key, entry);
        }
    }

    /// Stores a successfully loaded value in the cache for a given key.
    pub fn store(&self, key: K, value: V) {
        self.insert(key, CacheEntry::Loaded(Arc::new(value)));
    }

    /// Stores a successfully loaded value in the cache for a given key.
    pub fn store_idx(&self, key: K, headers: HeaderMap, value: V) -> Result<(), CacheError> {
        let vary = Self::resolve_vary(&headers)?;

        let cache_control = headers
            .get(CACHE_CONTROL)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .parse::<CacheControlResponse>()
            .unwrap_or_default();

        if cache_control.no_store {
            return Err(CacheError::WriteError(String::from(
                "Cache-Control: no-store prevents caching",
            )));
        }

        let sha = Self::hash_url(key.as_ref(), &vary);

        let data = match to_stdvec(&value) {
            Ok(d) => d,
            Err(_) => {
                return Err(CacheError::WriteError(String::from(
                    "Failed to serialize value",
                )));
            }
        };

        let header = CacheHeader::new(data.as_slice(), sha, &vary, &headers, &cache_control);

        if data.len() >= MAX_BLOCK_SIZE as usize {
            LargeFile::write(data.as_slice(), sha, header)
        } else {
            BlockFile::write(value, sha, header)
        }
    }

    /// Marks a key as pending if it is not already in the cache.
    pub fn mark_pending(&self, key: K) -> bool {
        let mut entries = self.entries.write().expect("Cache lock poisoned");
        if entries.contains_key(&key) {
            return false;
        }
        entries.insert(key, CacheEntry::Pending);
        true
    }

    /// Marks a key as failed.
    pub fn mark_failed(&self, key: K) {
        let mut entries = self.entries.write().expect("Cache lock poisoned");
        entries.insert(key, CacheEntry::Failed);
    }

    pub fn hash_url(url: &str, vary: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        hasher.update(vary.as_bytes());
        hasher.finalize().into()
    }

    fn resolve_vary(headers: &HeaderMap) -> Result<String, CacheError> {
        let vary = headers
            .get(VARY)
            .map(|v| v.to_str().unwrap_or_default())
            .unwrap_or_default();

        if vary.eq_ignore_ascii_case("*") {
            return Err(CacheError::WriteError(String::from("")));
        }

        let mut parts = Vec::new();

        for header in vary.split(',').map(|s| s.trim()) {
            let name = header.parse::<HeaderName>().map_err(|_| {
                CacheError::WriteError(format!("Invalid header name in Vary: '{}'", header))
            })?;

            let value = headers
                .get(&name)
                .ok_or_else(|| {
                    CacheError::WriteError(format!("Missing header '{}' specified in Vary", name))
                })?
                .to_str()
                .map_err(|_| {
                    CacheError::WriteError(format!("Invalid header value for '{}'", name))
                })?;

            parts.push(format!("{}:{}", name, value));
        }

        parts.sort();

        Ok(parts.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_get() {
        let mut headers = HeaderMap::new();
        headers.insert(VARY, "Accept-Encoding".parse().unwrap());
        let cache = Cache::<String, String>::new();
        let key = "https://example.com/resource".to_string();
        let value = "cached data".to_string();

        let result = cache.store_idx(key.clone(), headers.clone(), value.clone());
        assert!(result.is_ok());

        let mut headers = HeaderMap::new();
        headers.insert(VARY, "Accept-Encoding".parse().unwrap());
        headers.insert("Accept-Encoding", "gzip".parse().unwrap());

        let retrieved = cache.get_idx(&key, &headers).unwrap();
        assert_eq!(retrieved, value);
    }
}
