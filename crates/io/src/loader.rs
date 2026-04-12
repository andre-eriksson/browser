use std::path::Path;

use storage::paths::create_paths;

use crate::{
    Entry,
    embeded::{EmbededResource, EmbededType},
    errors::ResourceError,
    manager::ResourceType,
};

pub trait Loader {
    fn load_asset(self) -> Result<Vec<u8>, ResourceError>;
}

pub trait Writer {
    fn write<C: AsRef<[u8]>>(self, data: C) -> Result<(), ResourceError>;
}

impl<'path> Loader for ResourceType<'path> {
    fn load_asset(self) -> Result<Vec<u8>, ResourceError> {
        match self {
            ResourceType::Path(entry) => {
                let dir = entry
                    .path()
                    .ok_or_else(|| ResourceError::InvalidPath(entry.location().to_string()))?;

                if !dir.is_file() {
                    return Err(ResourceError::NotFound(entry.location().to_string()));
                }

                std::fs::read(dir).map_err(|_| ResourceError::NotFound(entry.location().to_string()))
            }
            ResourceType::Embeded(asset) => EmbededResource::get(&asset.path())
                .map(|file| file.data.into_owned())
                .ok_or_else(|| ResourceError::NotFound(asset.path())),
            ResourceType::Absolute { protocol, location } => match protocol {
                "file" => Self::load_asset(ResourceType::Path(Entry::absolute(location))),
                "embed" => Self::load_asset(ResourceType::Embeded(EmbededType::Root(location))),
                "about" => {
                    let adjusted_location = location.trim_start_matches("about:");
                    Self::load_asset(ResourceType::Embeded(EmbededType::Browser(adjusted_location)))
                }
                _ => Err(ResourceError::UnsupportedProtocol(protocol.to_string())),
            },
        }
    }
}

impl<'path> Writer for ResourceType<'path> {
    fn write<C: AsRef<[u8]>>(self, data: C) -> Result<(), ResourceError> {
        match self {
            ResourceType::Absolute { .. } | ResourceType::Embeded(_) => Err(ResourceError::UnsupportedOperation(
                "Cannot create or modify embedded or absolute resources".to_string(),
            )),
            ResourceType::Path(file) => {
                let path = file
                    .path()
                    .ok_or_else(|| ResourceError::InvalidPath(file.location().to_string()))?;

                if !is_relative_path(file.location()) {
                    return Err(ResourceError::InvalidPath(file.location().to_string()));
                }

                create_paths(&path.parent().unwrap().to_path_buf())
                    .map_err(|error| ResourceError::Io(error.to_string()))?;
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
