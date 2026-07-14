pub mod embeded;
pub mod errors;
pub mod files;
mod loader;
mod manager;

pub use files::Entry;
pub use manager::{Resource, ResourceType};
