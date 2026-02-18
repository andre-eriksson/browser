//! This module defines constants for file names used in the browser's cache and configuration directories.
//! These constants are of type `ResourceType`, which is an enum that categorizes resources based on their
//! storage location (cache, config, user data). The constants defined in this module provide a standardized
//! way to reference specific files used by the browser, such as user agent stylesheets and user preferences.

use crate::ResourceType;

/// The cache file name for user agent stylesheets.
/// This file is stored in the cache directory and contains precompiled stylesheets for user agent (browser default) styles.
pub const CACHE_USER_AGENT: ResourceType = ResourceType::Cache("stylesheets/useragent.bin");

/// The user preferences file name. This file is stored in the config directory and contains user-specific settings for the browser.
pub const PREFERENCES: ResourceType = ResourceType::Config("preferences.toml");
