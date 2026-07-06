use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use storage::{Directory, create_paths};

use crate::{
    Entry,
    embeded::{EmbededResource, EmbededType},
    errors::ResourceError,
    files::FilePath,
    manager::ResourceType,
};

pub trait Loader {
    fn load_asset(self, dirs: Option<Directory>, max_file_size: Option<u64>) -> Result<Vec<u8>, ResourceError>;
}

pub trait Writer {
    fn write<C: AsRef<[u8]>>(self, data: C, dirs: Directory) -> Result<(), ResourceError>;
}

impl Loader for ResourceType<'_> {
    fn load_asset(self, dirs: Option<Directory>, max_file_size: Option<u64>) -> Result<Vec<u8>, ResourceError> {
        match self {
            ResourceType::Path(entry) => {
                let dirs = dirs.ok_or_else(|| ResourceError::InvalidPath(entry.location().to_string()))?;

                let path = if entry.is_global() {
                    match entry.file_path() {
                        FilePath::Absolute => PathBuf::from(entry.location()),
                        FilePath::Cache => dirs.global_cache.join(entry.location()),
                        FilePath::Config => dirs.global_config.join(entry.location()),
                        FilePath::UserData => dirs.global_data.join(entry.location()),
                        FilePath::Temporary => dirs.temp.join(entry.location()),
                    }
                } else {
                    match entry.file_path() {
                        FilePath::Absolute => PathBuf::from(entry.location()),
                        FilePath::Cache => dirs.profile_cache.join(entry.location()),
                        FilePath::Config => dirs.profile_config.join(entry.location()),
                        FilePath::UserData => dirs.profile_data.join(entry.location()),
                        FilePath::Temporary => dirs.temp.join(entry.location()),
                    }
                };

                let mut file = File::open(path).map_err(|e| ResourceError::Io(e.to_string()))?;
                let metadata = file
                    .metadata()
                    .map_err(|e| ResourceError::Io(e.to_string()))?;

                if !metadata.is_file() {
                    return Err(ResourceError::NotFound(entry.location().to_string()));
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

                Ok(buffer)
            }
            ResourceType::Embeded(asset) => EmbededResource::get(&asset.path())
                .map(|file| file.data.into_owned())
                .ok_or_else(|| ResourceError::NotFound(asset.path())),
            ResourceType::Absolute { protocol, location } => match protocol {
                "file" => Self::load_asset(ResourceType::Path(Entry::absolute(location)), dirs, max_file_size),
                "embed" => Self::load_asset(ResourceType::Embeded(EmbededType::Root(location)), dirs, max_file_size),
                "about" => {
                    let adjusted_location = location.trim_start_matches("about:");
                    Self::load_asset(
                        ResourceType::Embeded(EmbededType::Browser(adjusted_location)),
                        dirs,
                        max_file_size,
                    )
                }
                _ => Err(ResourceError::UnsupportedProtocol(protocol.to_string())),
            },
        }
    }
}

impl Writer for ResourceType<'_> {
    fn write<C: AsRef<[u8]>>(self, data: C, dirs: Directory) -> Result<(), ResourceError> {
        match self {
            ResourceType::Absolute { .. } | ResourceType::Embeded(_) => Err(ResourceError::UnsupportedOperation(
                "Cannot create or modify embedded or absolute resources".to_string(),
            )),
            ResourceType::Path(entry) => {
                let path = if entry.is_global() {
                    match entry.file_path() {
                        FilePath::Absolute => PathBuf::from(entry.location()),
                        FilePath::Cache => dirs.global_cache.join(entry.location()),
                        FilePath::Config => dirs.global_config.join(entry.location()),
                        FilePath::UserData => dirs.global_data.join(entry.location()),
                        FilePath::Temporary => dirs.temp.join(entry.location()),
                    }
                } else {
                    match entry.file_path() {
                        FilePath::Absolute => PathBuf::from(entry.location()),
                        FilePath::Cache => dirs.profile_cache.join(entry.location()),
                        FilePath::Config => dirs.profile_config.join(entry.location()),
                        FilePath::UserData => dirs.profile_data.join(entry.location()),
                        FilePath::Temporary => dirs.temp.join(entry.location()),
                    }
                };

                if !is_relative_path(entry.location()) {
                    return Err(ResourceError::InvalidPath(entry.location().to_string()));
                }

                let Some(parent) = path.parent() else {
                    return Err(ResourceError::InvalidPath(entry.location().to_string()));
                };

                if let Err(error) = create_paths(&parent.to_path_buf()) {
                    return Err(ResourceError::Io(error.to_string()));
                }

                std::fs::write(path, data).map_err(|error| ResourceError::Io(error.to_string()))
            }
        }
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
