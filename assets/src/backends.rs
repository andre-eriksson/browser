use rust_embed::RustEmbed;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "assets/"]
#[include = "**/*"]
struct Asset;

pub trait AssetBackend {
    fn load_asset(&self, key: &str) -> Option<Vec<u8>>;
}

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
