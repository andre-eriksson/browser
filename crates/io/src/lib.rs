pub mod embed;
pub mod embedded;
pub mod entries;
pub mod entry;
pub mod errors;
pub mod http;
mod loader;
mod manager;

pub use entry::Entry;
pub use loader::{Loadable, Writable};
pub use manager::Resource;
