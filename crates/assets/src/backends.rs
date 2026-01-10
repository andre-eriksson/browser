use rust_embed::RustEmbed;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "resources/"]
#[include = "**/*"]
struct Asset;

/// AssetError represents errors that can occur when loading assets.
pub enum AssetError {
    /// Represents an error when an asset is not found.
    NotFound(String),

    /// Represents an error when an asset fails to load.
    LoadFailed(String),
}

/// AssetBackend is a trait that defines the behavior of an asset backend.
pub trait AssetBackend {
    /// Loads an asset by its key.
    ///
    /// # Arguments
    /// * `key` - The key of the asset to load, which can be a file path or identifier.
    ///
    /// # Returns
    /// An `Result<Vec<u8>>` containing the asset data or an error if the asset could not be loaded.
    fn load_asset(&self, key: &str) -> Result<Vec<u8>, AssetError>;
}

/// Backend represents different types of asset backends that can be used to load assets.
/// It can be a file system path, an embedded asset.
#[derive(PartialEq, Eq)]
pub enum Backend {
    /// Represents a file system backend with a specified path.
    ///
    /// Used for files that don't need the program to be compiled again e.g., config files.
    FileSystem(PathBuf),

    /// Represents the apps default assets embedded in the binary.
    ///
    /// Used for assets that are unlikely to change frequently e.g., UI assets, logos, etc.
    Embedded,
}

impl AssetBackend for Backend {
    fn load_asset(&self, key: &str) -> Result<Vec<u8>, AssetError> {
        match self {
            Backend::FileSystem(path) => {
                let full_path = path.join(key);
                std::fs::read(full_path).map_err(|_| AssetError::LoadFailed(key.to_string()))
            }
            Backend::Embedded => Asset::get(key)
                .map(|file| file.data.into_owned())
                .ok_or_else(|| AssetError::NotFound(key.to_string())),
        }
    }
}
