//! Storage crate for the browser.
//!
//! This crate contains useful utilities for getting paths to important directories and files, such as cache, config, and user data directories.

mod paths;

pub use paths::{create_paths, get_cache_path, get_config_path, get_data_path, get_temp_path};
