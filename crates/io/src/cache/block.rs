use std::{
    fs::{self, File, OpenOptions},
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    path::Path,
};

use serde::{Serialize, de::DeserializeOwned};

use storage::paths::get_cache_path;

use crate::cache::{
    header::CacheHeader,
    index::{BlockPointer, IndexFile, Pointer},
};

const BLOCK_DIR: &str = "blocks";
const MAX_BLOCK_SIZE: u64 = 20_000_000; // 20 MB

pub struct BlockFile;

impl BlockFile {
    pub fn write<V>(value: V, sha: [u8; 32], header: CacheHeader)
    where
        V: Clone + Serialize + DeserializeOwned,
    {
        let count = Self::count_bin_files(format!(
            "{}/{}",
            get_cache_path().unwrap().to_str().unwrap(),
            BLOCK_DIR,
        ))
        .unwrap_or(0);

        let data_path = format!(
            "{}/{}/{}.bin",
            get_cache_path().unwrap().to_str().unwrap(),
            BLOCK_DIR,
            count.saturating_sub(1)
        );

        let _ = fs::create_dir_all(format!(
            "{}/{}",
            get_cache_path().unwrap().to_str().unwrap(),
            BLOCK_DIR,
        ));

        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&data_path);

        // TODO: Better error handling and cleanup on failure
        if file.is_err() {
            return;
        }

        let header_bytes = postcard::to_stdvec(&header).ok();
        let data_bytes = postcard::to_stdvec(&value).ok();

        if header_bytes.is_none() || data_bytes.is_none() {
            return;
        }
        let header_bytes = header_bytes.unwrap();
        let data_bytes = data_bytes.unwrap();

        let total_length = (header_bytes.len() + data_bytes.len()) as u32;

        let mut header = header;
        header.content_size = total_length;

        let mut file = file.unwrap();

        if file.metadata().map(|m| m.len()).unwrap_or(0) + header.content_size as u64
            > MAX_BLOCK_SIZE
        {
            return;
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

        idx_file.entries.insert(sha, Pointer::Block(pointer));
        let _ = idx_file.write();
    }

    pub fn read<V>(pointer: &BlockPointer) -> Option<(CacheHeader, V)>
    where
        V: Clone + Serialize + DeserializeOwned,
    {
        let data_path = format!(
            "{}/{}/{}.bin",
            get_cache_path().unwrap().to_str().unwrap(),
            BLOCK_DIR,
            pointer.block_id.saturating_sub(1)
        );

        let mut file = File::open(&data_path).ok()?;
        file.seek(SeekFrom::Start(pointer.offset as u64)).ok()?;

        let mut header_buf = vec![0u8; pointer.header_size as usize];
        file.read_exact(&mut header_buf).ok()?;
        let header: CacheHeader = postcard::from_bytes(&header_buf).ok()?;

        let mut data_buf = vec![0u8; pointer.content_size as usize];
        file.read_exact(&mut data_buf).ok()?;
        let data: V = postcard::from_bytes(&data_buf).ok()?;

        Some((header, data))
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
