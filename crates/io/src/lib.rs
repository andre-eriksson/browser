pub mod embeded;
pub mod errors;
pub mod files;
pub mod http;
mod loader;
mod manager;

pub use files::Entry;
pub use loader::{Loadable, Writable};
pub use manager::Resource;
