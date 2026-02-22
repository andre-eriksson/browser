use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use postcard::{from_bytes, to_stdvec};
use serde::{Deserialize, Serialize};

use crate::{Resource, ResourceType, cache::errors::CacheError};

const IDX_MAGIC: [u8; 4] = *b"CIDX";
const IDX_VERSION: u16 = 1;
const IDX_FILE_PATH: &str = "cache/index.idx";

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexFile {
    pub magic: [u8; 4],
    pub version: u16,
    pub created_at: u64,
    pub last_compacted_at: u64,
    pub entries: HashMap<[u8; 32], Pointer>,
}

impl Default for IndexFile {
    fn default() -> Self {
        Self {
            magic: IDX_MAGIC,
            version: IDX_VERSION,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            last_compacted_at: 0,
            entries: HashMap::new(),
        }
    }
}

impl IndexFile {
    pub fn load() -> Option<Self> {
        let idx_file = Resource::load(ResourceType::Cache(IDX_FILE_PATH)).ok()?;

        from_bytes(&idx_file).ok()
    }

    pub fn write(&self) -> Result<(), CacheError> {
        let idx_data = to_stdvec(self).map_err(CacheError::SerializationError)?;

        Resource::write(ResourceType::Cache(IDX_FILE_PATH), idx_data)
            .map_err(CacheError::AssetError)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Pointer {
    Block(BlockPointer),
    Large,
}

/// A pointer to a block of data in the cache file, including its location and status.
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockPointer {
    pub block_id: u32,
    pub offset: u32,
    pub header_size: u32,
    pub content_size: u32,
    pub dead: bool,
}
