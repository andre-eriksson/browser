use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use database::Database;
use storage::paths::get_cache_path;

use crate::cache::{
    errors::CacheError,
    header::CacheHeader,
    index::{IndexDatabase, IndexTable},
};

/// The magic bytes and version number at the start of each block file, used to validate the file format when reading.
const MAGIC: [u8; 4] = *b"BLKC";
/// The current version of the block file format. Increment this if any incompatible changes are made to the
/// structure of block files, so that readers can detect unsupported formats and avoid misinterpreting data.
/// This is separate from the header version in CacheHeader, which tracks the format of individual entries
/// rather than the overall block file structure.
const VERSION: u16 = 1;
/// Directory within the cache path where block files are stored. Each block file contains multiple cache
/// entries, allowing for more efficient storage of small resources and better space utilization compared
/// to storing each entry
const BLOCK_DIR: &str = "resources/blocks";
/// 20 MB - This threshold determines whether a cache entry is stored as a block or as a large file.
/// Entries larger than this size will be stored as large files, while smaller entries will be
/// stored in block files.
pub const MAX_BLOCK_SIZE: u64 = 1000;
/// Minimum file fullness ratio before compaction is considered (80%).
const COMPACTION_FULLNESS_THRESHOLD: f64 = 0.80;
/// Minimum dead-byte ratio within entries to trigger compaction (50%).
const COMPACTION_DEAD_THRESHOLD: f64 = 0.50;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    pub magic: [u8; 4],
    pub version: u16,
}

pub struct BlockFile;

impl BlockFile {
    pub fn write(
        value: &[u8],
        header: &mut CacheHeader,
    ) -> Result<(u32, u32, u32, u32), CacheError> {
        let cache_path = match get_cache_path() {
            Some(path) => path,
            None => return Err(CacheError::WriteError(String::from("Cache path not found"))),
        };

        let block_dir = cache_path.join(BLOCK_DIR);
        fs::create_dir_all(&block_dir)?;

        let (path, file_number) = Self::find_writable_file(&block_dir)?;

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

        let mut file = match OpenOptions::new().append(true).create(true).open(&path) {
            Ok(file) => file,
            Err(e) => return Err(CacheError::IoError(e)),
        };

        let metadata = file.metadata().map_err(CacheError::IoError)?;
        let file_len = metadata.len();

        if file_len == 0 {
            file.write_all(&block_header_bytes)?;
        } else if file_len < block_header_bytes.len() as u64 {
            return Err(CacheError::CorruptedBlock);
        }

        let data_bytes = value.to_vec();
        header.content_size = data_bytes.len() as u32;

        let header_bytes = match postcard::to_stdvec(&header) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Err(CacheError::WriteError(format!(
                    "Failed to serialize header: {}",
                    e
                )));
            }
        };

        let offset = match file.stream_position() {
            Ok(pos) => {
                if pos == 0 {
                    block_header_bytes.len() as u64
                } else {
                    pos
                }
            }
            Err(_) => block_header_bytes.len() as u64,
        } as u32;

        file.write_all(&header_bytes)?;
        file.write_all(&data_bytes)?;
        file.flush()?;

        let file_id = file_number + 1;

        Ok((
            file_id,
            offset,
            header_bytes.len() as u32,
            data_bytes.len() as u32,
        ))
    }

    pub fn read(
        block_id: u32,
        offset: u32,
        header_size: u32,
        content_size: u32,
    ) -> Result<(CacheHeader, Vec<u8>, usize), CacheError> {
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

        Ok((header, data_buf, content_size as usize))
    }

    pub fn delete(block_id: u32, offset: u32, header_size: u32) -> Result<(), CacheError> {
        let data_path = format!(
            "{}/{}/{}.bin",
            get_cache_path().unwrap().to_str().unwrap(),
            BLOCK_DIR,
            block_id.saturating_sub(1)
        );

        let mut file = OpenOptions::new().read(true).write(true).open(&data_path)?;
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

    /// Compacts block files on a per-file basis.
    ///
    /// For each `.bin` file in the blocks directory that is nearly full (>= 80% of
    /// `MAX_BLOCK_SIZE`) and has a high proportion of dead entries (>= 50% of entry bytes),
    /// this rewrites the file with only the live entries, reclaiming space.
    ///
    /// After compaction the index database is updated with the new offsets so that
    /// subsequent reads resolve correctly. Dead entries are also pruned from the index.
    pub fn compact() -> Result<(), CacheError> {
        let cache_path = match get_cache_path() {
            Some(path) => path,
            None => return Err(CacheError::WriteError(String::from("Cache path not found"))),
        };

        let block_dir = cache_path.join(BLOCK_DIR);
        if !block_dir.exists() {
            return Ok(());
        }

        let conn = IndexDatabase::open()
            .map_err(|e| CacheError::WriteError(format!("Failed to open index database: {}", e)))?;

        let mut files: Vec<(PathBuf, u32)> = fs::read_dir(&block_dir)?
            .flatten()
            .filter(|e| e.path().is_file() && e.path().extension().is_some_and(|ext| ext == "bin"))
            .filter_map(|e| {
                let path = e.path();
                let num: u32 = path.file_stem()?.to_str()?.parse().ok()?;
                Some((path, num))
            })
            .collect();

        files.sort_by_key(|(_, num)| *num);

        for (path, _file_number) in &files {
            let metadata = fs::metadata(path)?;
            let file_size = metadata.len();

            if (file_size as f64) < (MAX_BLOCK_SIZE as f64 * COMPACTION_FULLNESS_THRESHOLD) {
                continue;
            }

            let data = fs::read(path)?;

            let (block_header, remaining): (BlockHeader, &[u8]) =
                postcard::take_from_bytes(&data).map_err(|_| CacheError::CorruptedBlock)?;

            if block_header.magic != MAGIC || block_header.version != VERSION {
                continue;
            }

            let block_header_size = data.len() - remaining.len();

            struct ScannedEntry {
                url_hash: [u8; 32],
                header_bytes: Vec<u8>,
                content_bytes: Vec<u8>,
                is_dead: bool,
            }

            let mut entries: Vec<ScannedEntry> = Vec::new();
            let mut total_entry_bytes: u64 = 0;
            let mut dead_entry_bytes: u64 = 0;
            let mut cursor: &[u8] = remaining;

            while !cursor.is_empty() {
                let (header, after_header): (CacheHeader, &[u8]) =
                    match postcard::take_from_bytes(cursor) {
                        Ok(result) => result,
                        Err(_) => break,
                    };

                let header_size = cursor.len() - after_header.len();
                let content_size = header.content_size as usize;

                if after_header.len() < content_size {
                    break;
                }

                let entry_size = (header_size + content_size) as u64;
                total_entry_bytes += entry_size;

                if header.dead {
                    dead_entry_bytes += entry_size;
                }

                entries.push(ScannedEntry {
                    url_hash: header.url_hash,
                    header_bytes: cursor[..header_size].to_vec(),
                    content_bytes: after_header[..content_size].to_vec(),
                    is_dead: header.dead,
                });

                cursor = &after_header[content_size..];
            }

            if total_entry_bytes == 0 {
                continue;
            }

            let dead_ratio = dead_entry_bytes as f64 / total_entry_bytes as f64;
            if dead_ratio < COMPACTION_DEAD_THRESHOLD {
                continue;
            }

            let temp_path = path.with_extension("tmp");

            {
                let mut new_file = File::create(&temp_path).map_err(CacheError::IoError)?;

                new_file.write_all(&data[..block_header_size])?;

                let mut new_offset = block_header_size as u32;

                for entry in &entries {
                    if entry.is_dead {
                        let _ = IndexTable::delete_by_key(&conn, &entry.url_hash);
                        continue;
                    }

                    new_file.write_all(&entry.header_bytes)?;
                    new_file.write_all(&entry.content_bytes)?;

                    let _ = IndexTable::update_block_offset(
                        &conn,
                        &entry.url_hash,
                        new_offset,
                        entry.header_bytes.len() as u32,
                    );

                    new_offset +=
                        entry.header_bytes.len() as u32 + entry.content_bytes.len() as u32;
                }

                new_file.flush()?;
            }

            fs::rename(&temp_path, path)?;
        }

        Ok(())
    }

    /// Finds the first block file that still has room for more data, or determines
    /// the path for a brand-new file if every existing file is full.
    ///
    /// Returns `(path, file_number)` where `file_number` is the numeric stem of the
    /// `.bin` file (e.g. `3` for `3.bin`).  The caller derives the persisted
    /// `file_id` as `file_number + 1` so that `read()` can recover the filename via
    /// `file_id.saturating_sub(1)`.
    fn find_writable_file(block_dir: &Path) -> Result<(PathBuf, u32), CacheError> {
        if !block_dir.exists() {
            return Ok((block_dir.join("0.bin"), 0));
        }

        let mut files: Vec<(PathBuf, u32)> = fs::read_dir(block_dir)?
            .flatten()
            .filter(|e| e.path().is_file() && e.path().extension().is_some_and(|ext| ext == "bin"))
            .filter_map(|e| {
                let path = e.path();
                let num: u32 = path.file_stem()?.to_str()?.parse().ok()?;
                Some((path, num))
            })
            .collect();

        if files.is_empty() {
            return Ok((block_dir.join("0.bin"), 0));
        }

        files.sort_by_key(|(_, num)| *num);

        for (path, num) in &files {
            let meta = fs::metadata(path).map_err(CacheError::IoError)?;
            if meta.len() < MAX_BLOCK_SIZE {
                return Ok((path.clone(), *num));
            }
        }

        let next_num = files.last().map(|(_, n)| n + 1).unwrap_or(0);
        Ok((block_dir.join(format!("{}.bin", next_num)), next_num))
    }
}
