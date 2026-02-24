use std::{
    fs::{self, File, OpenOptions},
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    path::Path,
};

use database::{Database, Table};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use storage::paths::get_cache_path;

use crate::cache::{
    errors::CacheError,
    header::CacheHeader,
    index::{Index, IndexDatabase, IndexEntry, IndexTable},
};

const MAGIC: [u8; 4] = *b"BLKC";
const VERSION: u16 = 1;
const BLOCK_DIR: &str = "resources/blocks";
const MAX_BLOCK_SIZE: u64 = 20_000_000; // 20 MB

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    pub magic: [u8; 4],
    pub version: u16,
}

pub struct BlockFile;

impl BlockFile {
    pub fn write<V>(value: V, sha: [u8; 32], header: CacheHeader) -> Result<(), CacheError>
    where
        V: Clone + Serialize + DeserializeOwned,
    {
        let cache_path = match get_cache_path() {
            Some(path) => path,
            None => return Err(CacheError::WriteError(String::from("Cache path not found"))),
        };

        let count = Self::count_bin_files(cache_path.join(BLOCK_DIR)).unwrap_or(0);

        let path =
            cache_path
                .join(BLOCK_DIR)
                .join(format!("{}{}", count.saturating_sub(1), ".bin"));

        fs::create_dir_all(cache_path.join(BLOCK_DIR)).map_err(CacheError::IoError)?;

        let mut file = match OpenOptions::new().append(true).create(true).open(&path) {
            Ok(file) => file,
            Err(e) => return Err(CacheError::IoError(e)),
        };

        let block_header = BlockHeader {
            magic: MAGIC,
            version: VERSION,
        };

        let block_header_bytes = match postcard::to_stdvec(&block_header) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Err(CacheError::WriteError(format!(
                    "Failed to serialize block header: {}",
                    e
                )));
            }
        };

        let mut file = match file.metadata() {
            Ok(metadata) => {
                if metadata.len() == 0 {
                    file.write_all(&block_header_bytes)?;
                    file
                } else if metadata.len() < 6 {
                    return Err(CacheError::CorruptedBlock);
                } else if metadata.len() > MAX_BLOCK_SIZE {
                    let count = count + 1;

                    let path = cache_path.join(BLOCK_DIR).join(format!(
                        "{}{}",
                        count.saturating_sub(1),
                        ".bin"
                    ));

                    fs::create_dir_all(cache_path.join(BLOCK_DIR)).map_err(CacheError::IoError)?;

                    let mut file = match OpenOptions::new().append(true).create(true).open(&path) {
                        Ok(file) => file,
                        Err(e) => return Err(CacheError::IoError(e)),
                    };

                    file.write_all(&block_header_bytes)?;
                    file
                } else {
                    file
                }
            }
            Err(e) => return Err(CacheError::IoError(e)),
        };

        let header_bytes = match postcard::to_stdvec(&header) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Err(CacheError::WriteError(format!(
                    "Failed to serialize header: {}",
                    e
                )));
            }
        };

        let data_bytes = match postcard::to_stdvec(&value) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Err(CacheError::WriteError(format!(
                    "Failed to serialize data: {}",
                    e
                )));
            }
        };

        let mut header = header;
        header.content_size = data_bytes.len() as u32;

        let offset = match file.stream_position() {
            Ok(pos) => {
                if pos == 0 {
                    pos + block_header_bytes.len() as u64
                } else {
                    pos
                }
            }
            Err(_) => block_header_bytes.len() as u64,
        } as u32;

        let mut writer = BufWriter::new(file);
        writer.write_all(&header_bytes).ok();
        writer.write_all(&data_bytes).ok();
        writer.flush().ok();

        if let Ok(connection) = IndexDatabase::open() {
            let index = Index {
                key: sha,
                entry: IndexEntry::Block,
                file_id: count as u32,
                offset: Some(offset),
                header_size: Some(header_bytes.len() as u32),
                content_size: data_bytes.len() as u32,
                expires_at: header.expires_at,
                created_at: header.fetched_at,
                vary: header.vary.clone(),
            };

            let _ = IndexTable::insert(&connection, &index);
        }

        Ok(())
    }

    pub fn read<V>(
        block_id: u32,
        offset: u32,
        header_size: u32,
        content_size: u32,
    ) -> Result<(CacheHeader, V, usize), CacheError>
    where
        V: Clone + Serialize + DeserializeOwned,
    {
        let data_path = format!(
            "{}/{}/{}.bin",
            get_cache_path().unwrap().to_str().unwrap(),
            BLOCK_DIR,
            block_id.saturating_sub(1)
        );

        let mut file = File::open(&data_path)?;

        let block_header = BlockHeader {
            magic: MAGIC,
            version: VERSION,
        };

        let block_header_bytes = match postcard::to_stdvec(&block_header) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Err(CacheError::WriteError(format!(
                    "Failed to serialize block header: {}",
                    e
                )));
            }
        };

        let mut block_header_buf = vec![0u8; block_header_bytes.len()];
        file.read_exact(&mut block_header_buf)?;
        let read_block_header: BlockHeader =
            postcard::from_bytes(&block_header_buf).map_err(|_| CacheError::CorruptedBlock)?;

        if read_block_header.magic != MAGIC || read_block_header.version != VERSION {
            return Err(CacheError::CorruptedBlock);
        }

        file.seek(SeekFrom::Start(offset as u64))?;

        let mut header_buf = vec![0u8; header_size as usize];
        file.read_exact(&mut header_buf)?;
        let header: CacheHeader =
            postcard::from_bytes(&header_buf).map_err(|_| CacheError::CorruptedHeader)?;

        let mut data_buf = vec![0u8; content_size as usize];
        file.read_exact(&mut data_buf)?;
        let data: V = postcard::from_bytes(&data_buf).map_err(|_| CacheError::CorruptedBlock)?;

        Ok((header, data, content_size as usize))
    }

    pub fn delete(block_id: u32, offset: u32, header_size: u32) -> Result<(), CacheError> {
        let data_path = format!(
            "{}/{}/{}.bin",
            get_cache_path().unwrap().to_str().unwrap(),
            BLOCK_DIR,
            block_id.saturating_sub(1)
        );

        let mut file = OpenOptions::new().write(true).open(&data_path)?;
        file.seek(SeekFrom::Start(offset as u64))?;

        let mut header_buf = vec![0u8; header_size as usize];
        file.read_exact(&mut header_buf)?;
        let mut header: CacheHeader =
            postcard::from_bytes(&header_buf).map_err(|_| CacheError::CorruptedHeader)?;

        header.dead = true;

        let header_bytes = postcard::to_stdvec(&header)
            .map_err(|_| CacheError::WriteError(String::from("Failed to serialize header")))?;

        if header_bytes.len() as u32 != header_size {
            return Err(CacheError::WriteError(String::from(
                "Serialized header size mismatch",
            )));
        }

        file.seek(SeekFrom::Start(offset as u64))?;
        file.write_all(&header_bytes)?;

        Ok(())
    }

    fn count_bin_files<P: AsRef<Path>>(dir: P) -> std::io::Result<usize> {
        let count = fs::read_dir(dir)?
            .flatten()
            .filter(|entry| {
                entry.path().is_file() && entry.path().extension().is_some_and(|ext| ext == "bin")
            })
            .count();

        Ok(count)
    }
}
