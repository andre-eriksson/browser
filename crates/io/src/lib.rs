pub mod embed;
pub mod embedded;
pub mod entries;
pub mod entry;
pub mod errors;
pub mod http;
pub mod paths;
mod traits;

pub use entry::Entry;
pub use traits::{Readable, Writable};
