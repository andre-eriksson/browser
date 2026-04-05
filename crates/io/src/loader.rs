use std::path::Path;

use storage::paths::create_paths;

use crate::{
    embeded::{EmbededResource, EmbededType},
    errors::AssetError,
    manager::{Entry, FilePath, ResourceType},
};

pub trait Loader {
    fn load_asset(self) -> Result<Vec<u8>, AssetError>;
}

pub trait Writer {
    fn write<C: AsRef<[u8]>>(self, data: C) -> Result<(), AssetError>;
}

impl<'a> Loader for ResourceType<'a> {
    fn load_asset(self) -> Result<Vec<u8>, AssetError> {
        match self {
            ResourceType::Path(entry) => {
                let dir = entry
                    .path()
                    .ok_or_else(|| AssetError::InvalidPath(entry.path.to_string()))?;

                if !dir.is_file() {
                    return Err(AssetError::NotFound(entry.path.to_string()));
                }

                std::fs::read(dir).map_err(|_| AssetError::NotFound(entry.path.to_string()))
            }
            ResourceType::Embeded(asset) => EmbededResource::get(&asset.path())
                .map(|file| file.data.into_owned())
                .ok_or_else(|| AssetError::NotFound(asset.path())),
            ResourceType::Absolute { protocol, location } => match protocol {
                "file" => Self::load_asset(ResourceType::Path(Entry {
                    file_path: FilePath::Absolute,
                    path: location,
                })),
                "embed" => Self::load_asset(ResourceType::Embeded(EmbededType::Root(location))),
                "about" => {
                    let adjusted_location = location.trim_start_matches("about:");
                    Self::load_asset(ResourceType::Embeded(EmbededType::Browser(adjusted_location)))
                }
                _ => Err(AssetError::UnsupportedProtocol(protocol.to_string())),
            },
        }
    }
}

impl<'a> Writer for ResourceType<'a> {
    fn write<C: AsRef<[u8]>>(self, data: C) -> Result<(), AssetError> {
        match self {
            ResourceType::Absolute { .. } | ResourceType::Embeded(_) => Err(AssetError::UnsupportedOperation(
                "Cannot create or modify embedded or absolute resources".to_string(),
            )),
            ResourceType::Path(file) => {
                let path = file
                    .path()
                    .ok_or_else(|| AssetError::InvalidPath(file.path.to_string()))?;

                if !is_relative_path(file.path) {
                    return Err(AssetError::InvalidPath(file.path.to_string()));
                }

                create_paths(&path.parent().unwrap().to_path_buf())
                    .map_err(|_| AssetError::WriteFailed(file.path.to_string()))?;
                std::fs::write(path, data).map_err(|_| AssetError::WriteFailed(file.path.to_string()))
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
