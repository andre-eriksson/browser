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
    disk::DiskCache,
    errors::{CacheError, CacheRead},
    header::{CacheControlResponse, CacheHeader},
};

/// The current state of a cached resource in memory.
#[derive(Debug, Clone)]
pub enum CacheEntry<T: Clone> {
    /// The resource has been requested but not yet loaded.
    Pending,
    /// The resource has been successfully loaded.
    Loaded(Arc<CacheRead<T>>),
    /// The resource failed to load.
    Failed,
}

/// A thread-safe cache for resources, keyed by a generic key type `K`.
#[derive(Debug, Clone)]
pub struct MemoryCache<K, V: Clone> {
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
}

impl Default for MemoryCache<String, Vec<u8>> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> MemoryCache<K, V>
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
    pub fn get(&self, key: &K, headers: &HeaderMap) -> Result<CacheEntry<V>, CacheError> {
        let entries = self.entries.read().expect("Cache lock poisoned");
        if let Some(entry) = entries.get(key) {
            return Ok(entry.clone());
        }
        drop(entries);

        self.get_from_disk(key, headers)?
            .map_or(Ok(CacheEntry::Pending), |value| {
                Ok(CacheEntry::Loaded(Arc::new(CacheRead::Hit(value))))
            })
    }

    fn get_from_disk(&self, key: &K, headers: &HeaderMap) -> Result<Option<V>, CacheError> {
        let vary = Self::resolve_vary(headers)?;
        let sha = Self::hash_url(key.as_ref(), &vary);

        if let Some(data) = DiskCache::get(sha)? {
            let deserialized: V = postcard::from_bytes(&data).map_err(|_| {
                CacheError::ReadError(String::from("Failed to deserialize cache data"))
            })?;
            Ok(Some(deserialized))
        } else {
            Ok(None)
        }
    }

    /// Stores a successfully loaded value in the cache for a given key.
    pub fn store(&self, key: K, value: V, headers: &HeaderMap) -> Result<(), CacheError> {
        if let Ok(entries) = self.entries.write()
            && let Some(existing) = entries.get(&key)
        {
            match existing {
                CacheEntry::Loaded(_) => {
                    return Err(CacheError::WriteError(String::from(
                        "Cache entry already exists for this key",
                    )));
                }
                _ => { /* Allow overwriting Pending or Failed entries */ }
            }
        }

        self.store_in_disk(&key, &value, headers)?;
        self.insert(key, CacheEntry::Loaded(Arc::new(CacheRead::Hit(value))));

        Ok(())
    }

    fn store_in_disk(&self, key: &K, value: &V, headers: &HeaderMap) -> Result<(), CacheError> {
        let vary = Self::resolve_vary(headers)?;
        let sha = Self::hash_url(key.as_ref(), &vary);

        let serialized = to_stdvec(value)
            .map_err(|_| CacheError::WriteError(String::from("Failed to serialize cache data")))?;

        let cache_control = CacheControlResponse::from(
            headers
                .get(CACHE_CONTROL)
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default(),
        );

        if cache_control.no_store {
            return Err(CacheError::WriteError(String::from(
                "Cache-Control: no-store prevents caching",
            )));
        }

        let header = CacheHeader::new(serialized.as_slice(), sha, &vary, headers, &cache_control);

        DiskCache::put(sha, serialized.as_slice(), header)
    }

    /// Evicts a cache entry for a given key, removing it from both memory and disk.
    pub fn evict(&self, key: &K, headers: &HeaderMap) -> Result<bool, CacheError> {
        let mut removed_mem = false;

        if let Ok(mut entries) = self.entries.write() {
            removed_mem = entries.remove(key).is_some();
        }

        let vary = Self::resolve_vary(headers)?;
        let sha = Self::hash_url(key.as_ref(), &vary);

        let removed_disk = DiskCache::remove(sha, None)?;
        Ok(removed_mem || removed_disk)
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

    /// Inserts or updates a cache entry for a given key.
    fn insert(&self, key: K, entry: CacheEntry<V>) {
        if let Ok(mut entries) = self.entries.write() {
            entries.insert(key, entry);
        }
    }

    /// Hashes a URL and its Vary string to produce a unique key for caching.
    fn hash_url(url: &str, vary: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        hasher.update(vary.as_bytes());
        hasher.finalize().into()
    }

    /// Resolves the Vary header to determine which request headers affect the cache key.
    fn resolve_vary(headers: &HeaderMap) -> Result<String, CacheError> {
        let vary = headers
            .get(VARY)
            .map(|v| v.to_str().unwrap_or_default())
            .unwrap_or_default();

        if vary.eq_ignore_ascii_case("*") {
            return Err(CacheError::WriteError(String::from(
                "Vary: * prevents caching",
            )));
        } else if vary.is_empty() {
            return Ok(String::new());
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
        headers.insert("Accept-Encoding", "gzip".parse().unwrap());
        let cache = MemoryCache::<String, String>::new();
        let key = "https://example.com/resource".to_string();
        let value = "cached data".to_string();

        let result = cache.store_in_disk(&key, &value, &headers);
        assert!(result.is_ok());

        let mut headers = HeaderMap::new();
        headers.insert(VARY, "Accept-Encoding".parse().unwrap());
        headers.insert("Accept-Encoding", "gzip".parse().unwrap());

        let retrieved = cache.get_from_disk(&key, &headers).unwrap();
        assert!(retrieved.is_some());

        assert_eq!(value, retrieved.unwrap());
    }
}
