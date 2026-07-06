//! This module defines constants for file names used in the browser's cache and configuration directories.
//!
//! These constants are of type `ResourceType`, which is an enum that categorizes resources based on their
//! storage location (cache, config, user data). The constants defined in this module provide a standardized
//! way to reference specific files used by the browser, such as user agent stylesheets and user preferences.

use crate::ResourceType;

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

    /// Creates a new `Entry` for cache files. The file will be located in the cache directory, and the provided `path`
    /// will be appended to that directory.
    #[must_use]
    pub const fn cache(path: &'path str) -> Self {
        Self {
            location: path,
            file_path: FilePath::Cache,
        }
    }

    /// Creates a new `Entry` for configuration files. The file will be located in the configuration directory, and the
    /// provided `path` will be appended to that directory.
    #[must_use]
    pub const fn config(path: &'path str) -> Self {
        Self {
            location: path,
            file_path: FilePath::Config,
        }
    }

    /// Creates a new `Entry` for user data files. The file will be located in the user data directory, and the provided
    /// `path` will be appended to that directory.
    #[must_use]
    pub const fn user_data(path: &'path str) -> Self {
        Self {
            location: path,
            file_path: FilePath::UserData,
        }
    }

    /// Creates a new `Entry` for an absolute file path. The provided `path` should be an absolute path to the file.
    #[must_use]
    pub const fn absolute(path: &'path str) -> Self {
        Self {
            location: path,
            file_path: FilePath::Absolute,
        }
    }

    /// Creates a new `Entry` for a temporary file. The file will be located in the system's temporary directory, and the
    /// provided `path` will be appended to that directory.
    #[must_use]
    pub const fn temporary(path: &'path str) -> Self {
        Self {
            location: path,
            file_path: FilePath::Temporary,
        }
    }
}

/// The cache file name for user agent stylesheets.
/// This file is stored in the cache directory and contains precompiled stylesheets for user agent (browser default) styles.
pub const CACHE_USER_AGENT: ResourceType = ResourceType::Path(Entry::cache("stylesheets/useragent.bin"));

/// The user preferences file name. This file is stored in the config directory and contains user-specific settings for the browser.
pub const PREFERENCES: ResourceType = ResourceType::Path(Entry::config("preferences.toml"));
