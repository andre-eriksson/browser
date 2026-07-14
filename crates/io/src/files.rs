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
use storage::{Directory, create_paths};

use crate::{
    errors::ResourceError,
    loader::{Loadable, Writable},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilePath {
    Cache,
    Config,
    UserData,
    Absolute,
    Temporary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entry<'path> {
    location: &'path str,
    file_path: FilePath,
    global: bool,
}

impl<'path> Entry<'path> {
    /// Returns the original location string provided when creating the `Entry`. This is the relative path that will be
    /// appended to the base directory (cache, config, user data, or temporary) when resolving the full file path.
    #[must_use]
    pub const fn location(&self) -> &'path str {
        self.location
    }

    /// Returns the `FilePath` type of this `Entry`, which indicates where the file is located (cache, config, user data, absolute, or temporary).
    #[must_use]
    pub const fn file_path(&self) -> &FilePath {
        &self.file_path
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
            location: path,
            file_path: FilePath::Cache,
            global,
        }
    }

    /// Creates a new `Entry` for configuration files. The file will be located in the configuration directory, and the
    /// provided `path` will be appended to that directory.
    #[must_use]
    pub const fn config(path: &'path str, global: bool) -> Self {
        Self {
            location: path,
            file_path: FilePath::Config,
            global,
        }
    }

    /// Creates a new `Entry` for user data files. The file will be located in the user data directory, and the provided
    /// `path` will be appended to that directory.
    #[must_use]
    pub const fn user_data(path: &'path str, global: bool) -> Self {
        Self {
            location: path,
            file_path: FilePath::UserData,
            global,
        }
    }

    /// Creates a new `Entry` for an absolute file path. The provided `path` should be an absolute path to the file.
    #[must_use]
    pub const fn absolute(path: &'path str) -> Self {
        Self {
            location: path,
            file_path: FilePath::Absolute,
            global: false,
        }
    }

    /// Creates a new `Entry` for a temporary file. The file will be located in the system's temporary directory, and the
    /// provided `path` will be appended to that directory.
    #[must_use]
    pub const fn temporary(path: &'path str) -> Self {
        Self {
            location: path,
            file_path: FilePath::Temporary,
            global: false,
        }
    }
}

impl Loadable for Entry<'_> {
    type Output = Bytes;

    fn load_asset(
        self,
        dirs: &Directory,
        max_file_size: Option<u64>,
    ) -> Result<Self::Output, crate::errors::ResourceError> {
        let path = if self.is_global() {
            match self.file_path() {
                FilePath::Absolute => PathBuf::from(self.location()),
                FilePath::Cache => dirs.global_cache.join(self.location()),
                FilePath::Config => dirs.global_config.join(self.location()),
                FilePath::UserData => dirs.global_data.join(self.location()),
                FilePath::Temporary => dirs.temp.join(self.location()),
            }
        } else {
            match self.file_path() {
                FilePath::Absolute => PathBuf::from(self.location()),
                FilePath::Cache => dirs.profile_cache.join(self.location()),
                FilePath::Config => dirs.profile_config.join(self.location()),
                FilePath::UserData => dirs.profile_data.join(self.location()),
                FilePath::Temporary => dirs.temp.join(self.location()),
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

impl Writable for Entry<'_> {
    fn write_asset<C: AsRef<[u8]>>(self, data: C, dirs: &Directory) -> Result<(), ResourceError> {
        let path = if self.is_global() {
            match self.file_path() {
                FilePath::Absolute => PathBuf::from(self.location()),
                FilePath::Cache => dirs.global_cache.join(self.location()),
                FilePath::Config => dirs.global_config.join(self.location()),
                FilePath::UserData => dirs.global_data.join(self.location()),
                FilePath::Temporary => dirs.temp.join(self.location()),
            }
        } else {
            match self.file_path() {
                FilePath::Absolute => PathBuf::from(self.location()),
                FilePath::Cache => dirs.profile_cache.join(self.location()),
                FilePath::Config => dirs.profile_config.join(self.location()),
                FilePath::UserData => dirs.profile_data.join(self.location()),
                FilePath::Temporary => dirs.temp.join(self.location()),
            }
        };

        if !is_relative_path(self.location()) {
            return Err(ResourceError::InvalidPath(self.location().to_string()));
        }

        let Some(parent) = path.parent() else {
            return Err(ResourceError::InvalidPath(self.location().to_string()));
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

/// The cache file name for user agent stylesheets.
/// This file is stored in the cache directory and contains precompiled stylesheets for user agent (browser default) styles.
pub const PROFILE_CACHE_USER_AGENT: Entry = Entry::cache("stylesheets/useragent.bin", false);

/// The user preferences file name. This file is stored in the config directory and contains user-specific settings for the browser.
pub const PROFILE_PREFERENCES: Entry = Entry::config("preferences.toml", false);

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
