use std::path::Path;

use storage::paths::{create_paths, get_cache_path, get_config_path, get_data_path};

use crate::{
    embeded::{EmbededResource, EmbededType},
    errors::AssetError,
    manager::ResourceType,
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
            ResourceType::Cache(file_path) => {
                let cache_path = get_cache_path();

                match cache_path {
                    Some(path) => {
                        create_paths(&path).map_err(|_| AssetError::Unavailable("app cache".to_string()))?;

                        let full_path = path.join(file_path);
                        if !full_path.exists() {
                            return Err(AssetError::NotFound(full_path.to_string_lossy().to_string()));
                        }

                        std::fs::read(full_path).map_err(|_| AssetError::LoadFailed(file_path.to_string()))
                    }
                    None => Err(AssetError::Unavailable("cache".to_string())),
                }
            }
            ResourceType::Config(file_path) => {
                let config_path = get_config_path();

                match config_path {
                    Some(path) => {
                        create_paths(&path).map_err(|_| AssetError::Unavailable("app config".to_string()))?;

                        let full_path = path.join(file_path);
                        if !full_path.exists() {
                            return Err(AssetError::NotFound(full_path.to_string_lossy().to_string()));
                        }

                        std::fs::read(full_path).map_err(|_| AssetError::LoadFailed(file_path.to_string()))
                    }
                    None => Err(AssetError::Unavailable("config".to_string())),
                }
            }
            ResourceType::UserData(file_path) => {
                let user_data_path = get_data_path();

                match user_data_path {
                    Some(path) => {
                        create_paths(&path).map_err(|_| AssetError::Unavailable("user data".to_string()))?;

                        let full_path = path.join(file_path);
                        if !full_path.exists() {
                            return Err(AssetError::NotFound(full_path.to_string_lossy().to_string()));
                        }

                        std::fs::read(full_path).map_err(|_| AssetError::LoadFailed(file_path.to_string()))
                    }
                    None => Err(AssetError::Unavailable("user data".to_string())),
                }
            }
            ResourceType::FileSystem(path) => std::fs::read(path).map_err(|_| AssetError::LoadFailed(path.to_string())),
            ResourceType::Embeded(asset) => EmbededResource::get(&asset.path())
                .map(|file| file.data.into_owned())
                .ok_or_else(|| AssetError::NotFound(asset.path())),
            ResourceType::Absolute { protocol, location } => match protocol {
                "file" => Self::load_asset(ResourceType::FileSystem(location)),
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
            ResourceType::Cache(file_path) => {
                let cache_path = get_cache_path();

                if !is_relative_path(file_path) {
                    return Err(AssetError::InvalidPath(file_path.to_string()));
                }

                match cache_path {
                    Some(path) => {
                        let full_path = path.join(file_path);

                        create_paths(&full_path.parent().unwrap_or(&path).to_path_buf())
                            .map_err(|_| AssetError::Unavailable("app cache".to_string()))?;

                        std::fs::write(full_path, data.as_ref())
                            .map_err(|_| AssetError::WriteFailed(file_path.to_string()))
                    }
                    None => Err(AssetError::Unavailable("cache".to_string())),
                }
            }
            ResourceType::Config(file_path) => {
                let config_path = get_config_path();

                if !is_relative_path(file_path) {
                    return Err(AssetError::InvalidPath(file_path.to_string()));
                }

                match config_path {
                    Some(path) => {
                        let full_path = path.join(file_path);

                        create_paths(&full_path.parent().unwrap_or(&path).to_path_buf())
                            .map_err(|_| AssetError::Unavailable("app config".to_string()))?;

                        std::fs::write(full_path, data.as_ref())
                            .map_err(|_| AssetError::WriteFailed(file_path.to_string()))
                    }
                    None => Err(AssetError::Unavailable("config".to_string())),
                }
            }
            ResourceType::UserData(file_path) => {
                let user_data_path = get_data_path();

                if !is_relative_path(file_path) {
                    return Err(AssetError::InvalidPath(file_path.to_string()));
                }

                match user_data_path {
                    Some(path) => {
                        let full_path = path.join(file_path);

                        create_paths(&full_path.parent().unwrap_or(&path).to_path_buf())
                            .map_err(|_| AssetError::Unavailable("app user data".to_string()))?;

                        std::fs::write(full_path, data.as_ref())
                            .map_err(|_| AssetError::WriteFailed(file_path.to_string()))
                    }
                    None => Err(AssetError::Unavailable("user data".to_string())),
                }
            }
            ResourceType::FileSystem(path) => {
                std::fs::write(path, data.as_ref()).map_err(|_| AssetError::WriteFailed(path.to_string()))
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
