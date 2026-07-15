//! This module defines constants for file names used in the browser's cache and configuration directories.
//!
//! These constants are of type `ResourceType`, which is an enum that categorizes resources based on their
//! storage location (cache, config, user data). The constants defined in this module provide a standardized
//! way to reference specific files used by the browser, such as user agent stylesheets and user preferences.

use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use bytes::Bytes;

use crate::{
    errors::ResourceError,
    paths::{AppPaths, create_paths},
    traits::{Readable, Writable},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntryCategory {
    Cache,
    Config,
    UserData,
    Absolute,
    Temporary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entry<'path> {
    path: &'path str,
    category: EntryCategory,
    global: bool,
}

impl<'path> Entry<'path> {
    /// Returns the original location string provided when creating the `Entry`. This is the relative path that will be
    /// appended to the base directory (cache, config, user data, or temporary) when resolving the full file path.
    #[must_use]
    pub const fn path(&self) -> &'path str {
        self.path
    }

    /// Returns the `FilePath` type of this `Entry`, which indicates where the file is located (cache, config, user data, absolute, or temporary).
    #[must_use]
    pub const fn file_path(&self) -> &EntryCategory {
        &self.category
    }

    /// Returns whether this `Entry` is marked as global. A global entry is one that is shared across different profiles or instances of the application,
    /// rather than being specific to a single user or session.
    #[must_use]
    pub const fn is_global(&self) -> bool {
        self.global
    }

    /// Creates a new `Entry` for cache files. The file will be located in the cache directory, and the provided `path`
    /// will be appended to that directory.
    #[must_use]
    pub const fn cache(path: &'path str, global: bool) -> Self {
        Self {
            path,
            category: EntryCategory::Cache,
            global,
        }
    }

    /// Creates a new `Entry` for configuration files. The file will be located in the configuration directory, and the
    /// provided `path` will be appended to that directory.
    #[must_use]
    pub const fn config(path: &'path str, global: bool) -> Self {
        Self {
            path,
            category: EntryCategory::Config,
            global,
        }
    }

    /// Creates a new `Entry` for user data files. The file will be located in the user data directory, and the provided
    /// `path` will be appended to that directory.
    #[must_use]
    pub const fn user_data(path: &'path str, global: bool) -> Self {
        Self {
            path,
            category: EntryCategory::UserData,
            global,
        }
    }

    /// Creates a new `Entry` for an absolute file path. The provided `path` should be an absolute path to the file.
    #[must_use]
    pub const fn absolute(path: &'path str) -> Self {
        Self {
            path,
            category: EntryCategory::Absolute,
            global: false,
        }
    }

    /// Creates a new `Entry` for a temporary file. The file will be located in the system's temporary directory, and the
    /// provided `path` will be appended to that directory.
    #[must_use]
    pub const fn temporary(path: &'path str) -> Self {
        Self {
            path,
            category: EntryCategory::Temporary,
            global: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AppDirectory<'path>(pub Entry<'path>);

impl<'path> AppDirectory<'path> {
    pub fn load_dir(
        &self,
        paths: &AppPaths,
        max_files: Option<usize>,
        max_file_size: Option<u64>,
    ) -> Result<Vec<Vec<u8>>, ResourceError> {
        let entry = self.0;

        let path = if entry.is_global() {
            match entry.file_path() {
                EntryCategory::Absolute => PathBuf::from(entry.path()),
                EntryCategory::Cache => paths.global_cache.join(entry.path()),
                EntryCategory::Config => paths.global_config.join(entry.path()),
                EntryCategory::UserData => paths.global_data.join(entry.path()),
                EntryCategory::Temporary => paths.temp.join(entry.path()),
            }
        } else {
            match entry.file_path() {
                EntryCategory::Absolute => PathBuf::from(entry.path()),
                EntryCategory::Cache => paths.profile_cache.join(entry.path()),
                EntryCategory::Config => paths.profile_config.join(entry.path()),
                EntryCategory::UserData => paths.profile_data.join(entry.path()),
                EntryCategory::Temporary => paths.temp.join(entry.path()),
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

pub struct AppFile<'path>(pub Entry<'path>);

impl Readable for AppFile<'_> {
    type Output = Bytes;

    fn read(self, paths: &AppPaths, max_file_size: Option<u64>) -> Result<Self::Output, crate::errors::ResourceError> {
        let entry = self.0;

        let path = if entry.is_global() {
            match entry.file_path() {
                EntryCategory::Absolute => PathBuf::from(entry.path()),
                EntryCategory::Cache => paths.global_cache.join(entry.path()),
                EntryCategory::Config => paths.global_config.join(entry.path()),
                EntryCategory::UserData => paths.global_data.join(entry.path()),
                EntryCategory::Temporary => paths.temp.join(entry.path()),
            }
        } else {
            match entry.file_path() {
                EntryCategory::Absolute => PathBuf::from(entry.path()),
                EntryCategory::Cache => paths.profile_cache.join(entry.path()),
                EntryCategory::Config => paths.profile_config.join(entry.path()),
                EntryCategory::UserData => paths.profile_data.join(entry.path()),
                EntryCategory::Temporary => paths.temp.join(entry.path()),
            }
        };

        let mut file = File::open(&path).map_err(|e| ResourceError::Io(e.to_string()))?;
        let metadata = file
            .metadata()
            .map_err(|e| ResourceError::Io(e.to_string()))?;

        if !metadata.is_file() {
            return Err(ResourceError::NotFound(path.display().to_string()));
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

        Ok(buffer.into())
    }
}

impl Writable for AppFile<'_> {
    fn write<C: AsRef<[u8]>>(self, data: C, paths: &AppPaths) -> Result<(), ResourceError> {
        let entry = self.0;

        let path = if entry.is_global() {
            match entry.file_path() {
                EntryCategory::Absolute => PathBuf::from(entry.path()),
                EntryCategory::Cache => paths.global_cache.join(entry.path()),
                EntryCategory::Config => paths.global_config.join(entry.path()),
                EntryCategory::UserData => paths.global_data.join(entry.path()),
                EntryCategory::Temporary => paths.temp.join(entry.path()),
            }
        } else {
            match entry.file_path() {
                EntryCategory::Absolute => PathBuf::from(entry.path()),
                EntryCategory::Cache => paths.profile_cache.join(entry.path()),
                EntryCategory::Config => paths.profile_config.join(entry.path()),
                EntryCategory::UserData => paths.profile_data.join(entry.path()),
                EntryCategory::Temporary => paths.temp.join(entry.path()),
            }
        };

        if !is_relative_path(entry.path()) {
            return Err(ResourceError::InvalidPath(entry.path().to_string()));
        }

        let Some(parent) = path.parent() else {
            return Err(ResourceError::InvalidPath(entry.path().to_string()));
        };

        if let Err(error) = create_paths(&parent.to_path_buf()) {
            return Err(ResourceError::Io(error.to_string()));
        }

        std::fs::write(path, data).map_err(|error| ResourceError::Io(error.to_string()))
    }
}

/// Validates that a given path is a relative path that does not contain any components that
/// could lead to directory traversal (like `..`) or absolute paths.
/// Returns `true` if the path is a valid relative path, and `false` otherwise.
fn is_relative_path(path: &str) -> bool {
    let path = Path::new(path);

    if path.is_absolute() {
        return false;
    }

    let mut component_count = 0;
    for component in path.components() {
        match component {
            std::path::Component::Normal(_) | std::path::Component::CurDir => {
                component_count += 1;
            }
            _ => return false,
        }
    }

    component_count > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_relative_path() {
        assert!(is_relative_path("valid/path"));
        assert!(is_relative_path("another/valid/path"));
        assert!(!is_relative_path("../invalid/path"));
        assert!(!is_relative_path("/absolute/path"));
        assert!(!is_relative_path("invalid/../path"));
        assert!(!is_relative_path(""));
        assert!(is_relative_path("."));
    }
}
