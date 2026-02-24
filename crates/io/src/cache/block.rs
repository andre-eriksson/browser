use std::{
    fs::{self, File, OpenOptions},
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    path::Path,
};

use serde::{Serialize, de::DeserializeOwned};

use storage::paths::get_cache_path;

use crate::cache::{
    errors::CacheError,
    header::CacheHeader,
    index::{BlockPointer, IndexFile, PointerType},
};

const BLOCK_DIR: &str = "resources/blocks";
const MAX_BLOCK_SIZE: u64 = 20_000_000; // 20 MB

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

        let total_length = (header_bytes.len() + data_bytes.len()) as u32;

        let mut header = header;
        header.content_size = total_length;

        if file.metadata().map(|m| m.len()).unwrap_or(0) + header.content_size as u64
            > MAX_BLOCK_SIZE
        {
            return Err(CacheError::WriteError(String::from(
                "Block file size limit exceeded",
            )));
        }

        let offset = file.stream_position().unwrap_or(0) as u32;

        let mut writer = BufWriter::new(file);
        writer.write_all(&header_bytes).ok();
        writer.write_all(&data_bytes).ok();
        writer.flush().ok();

        let mut idx_file = IndexFile::load().unwrap_or_default();

        let pointer = BlockPointer {
            block_id: count as u32,
            offset,
            header_size: header_bytes.len() as u32,
            content_size: data_bytes.len() as u32,
            dead: false,
        };

        idx_file.entries.insert(sha, PointerType::Block(pointer));
        idx_file.write()?;

        Ok(())
    }

    pub fn read<V>(pointer: &BlockPointer) -> Result<(CacheHeader, V, usize), CacheError>
    where
        V: Clone + Serialize + DeserializeOwned,
    {
        let data_path = format!(
            "{}/{}/{}.bin",
            get_cache_path().unwrap().to_str().unwrap(),
            BLOCK_DIR,
            pointer.block_id.saturating_sub(1)
        );

        let mut file = File::open(&data_path)?;
        file.seek(SeekFrom::Start(pointer.offset as u64))?;

        let mut header_buf = vec![0u8; pointer.header_size as usize];
        file.read_exact(&mut header_buf)?;
        let header: CacheHeader =
            postcard::from_bytes(&header_buf).map_err(|_| CacheError::CorruptedHeader)?;

        let mut data_buf = vec![0u8; pointer.content_size as usize];
        file.read_exact(&mut data_buf)?;
        let data: V = postcard::from_bytes(&data_buf).map_err(|_| CacheError::CorruptedBlock)?;

        Ok((header, data, pointer.content_size as usize))
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
