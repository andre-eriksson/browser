use rust_embed::RustEmbed;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "assets/"]
#[include = "**/*"]
struct Asset;

/// AssetBackend is a trait that defines the behavior of an asset backend.
pub trait AssetBackend {
    /// Loads an asset by its key.
    ///
    /// # Arguments
    /// * `key` - The key of the asset to load, which can be a file path or identifier.
    ///
    /// # Returns
    /// An `Option<Vec<u8>>` containing the asset data if found, or `None` if not found.
    fn load_asset(&self, key: &str) -> Option<Vec<u8>>;
}

/// Backend represents different types of asset backends that can be used to load assets.
/// It can be a file system path, an embedded asset, or potentially other types in the
/// future (like network or database).
///
/// # Variants
/// * `FileSystem(PathBuf)` - Represents a file system backend with a specified path.
/// * `Embedded` - Represents an embedded asset backend that uses Rust's embed feature.
pub enum Backend {
    FileSystem(PathBuf),
    Embedded,
    // Network,
    // etc,
}

impl AssetBackend for Backend {
    fn load_asset(&self, key: &str) -> Option<Vec<u8>> {
        match self {
            Backend::FileSystem(path) => {
                let full_path = path.join(key);
                std::fs::read(full_path).ok()
            }
            Backend::Embedded => Asset::get(key).map(|file| file.data.into_owned()),
        }
    }
}
