use std::{fs::File, io::Read, path::PathBuf};

use storage::AppPaths;

use crate::entry::EntryCategory;

use crate::{Entry, errors::ResourceError};

/// `Resource` is responsible for managing and loading assets from various backends.
pub struct Resource;

impl Resource {
    /// Default maximum file size limit for loading resources, set to 10 MiB. This limit helps prevent excessive memory usage when loading large files.
    pub const DEFAULT_MAX_FILE_SIZE: Option<u64> = Some(10 * 1024 * 1024);

    /// Default maximum number of files to load from a directory, set to 100. This limit helps prevent performance issues when loading directories with a large number of files.
    pub const DEFAULT_MAX_FILES: Option<u64> = Some(100);

    /// Loads all files from a specified directory resource, returning their contents as a vector of byte vectors.
    ///
    /// # Args
    /// * `dir` - The directory resource to load, which should be a `ResourceType::Path` pointing to a directory.
    /// * `max_files` - An optional maximum number of files to load from the directory. If the directory contains more files than this limit, an error will be returned.
    ///   If `None`, there is no limit on the number of files.
    /// * `max_file_size` - An optional maximum file size limit in bytes for each file. If any loaded file exceeds this size, it will be skipped and a warning will be logged.
    ///   If `None`, there is no size limit for individual files.
    pub fn load_dir(
        dir: Entry,
        paths: &AppPaths,
        max_files: Option<usize>,
        max_file_size: Option<u64>,
    ) -> Result<Vec<Vec<u8>>, ResourceError> {
        let path = if dir.is_global() {
            match dir.file_path() {
                EntryCategory::Absolute => PathBuf::from(dir.path()),
                EntryCategory::Cache => paths.global_cache.join(dir.path()),
                EntryCategory::Config => paths.global_config.join(dir.path()),
                EntryCategory::UserData => paths.global_data.join(dir.path()),
                EntryCategory::Temporary => paths.temp.join(dir.path()),
            }
        } else {
            match dir.file_path() {
                EntryCategory::Absolute => PathBuf::from(dir.path()),
                EntryCategory::Cache => paths.profile_cache.join(dir.path()),
                EntryCategory::Config => paths.profile_config.join(dir.path()),
                EntryCategory::UserData => paths.profile_data.join(dir.path()),
                EntryCategory::Temporary => paths.temp.join(dir.path()),
            }
        };

        let mut paths = Vec::new();

        for entry in
            std::fs::read_dir(path).map_err(|_| ResourceError::NotFound("Directory doesn't exist".to_string()))?
        {
            if let Some(max) = max_files
                && paths.len() >= max
            {
                return Err(ResourceError::TooManyEntries(format!(
                    "Directory contains too many entries, which exceeds the limit of {max}"
                )));
            }

            let entry = entry.map_err(|_| ResourceError::NotFound("Entry doesn't exist".to_string()))?;
            paths.push(entry.path());
        }

        let mut files = Vec::new();

        for path in paths {
            let Ok(mut file) = File::open(path) else {
                continue;
            };

            let Ok(metadata) = file.metadata() else {
                continue;
            };

            if !metadata.is_file() {
                continue;
            }

            if let Some(max) = max_file_size
                && metadata.len() > max
            {
                return Err(ResourceError::FileTooLarge {
                    data_size: metadata.len(),
                    max_size: max,
                });
            }

            let mut buffer = Vec::with_capacity(metadata.len() as usize);
            file.read_to_end(&mut buffer)
                .map_err(|e| ResourceError::Io(e.to_string()))?;
            files.push(buffer);
        }

        Ok(files)
    }
}
