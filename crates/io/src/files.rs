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

/// The cache file name for user agent stylesheets.
/// This file is stored in the cache directory and contains precompiled stylesheets for user agent (browser default) styles.
pub const PROFILE_CACHE_USER_AGENT: ResourceType = ResourceType::Path(Entry::cache("stylesheets/useragent.bin", false));

/// The user preferences file name. This file is stored in the config directory and contains user-specific settings for the browser.
pub const PROFILE_PREFERENCES: ResourceType = ResourceType::Path(Entry::config("preferences.toml", false));
