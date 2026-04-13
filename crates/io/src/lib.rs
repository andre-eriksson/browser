mod cache;
pub mod embeded;
pub mod errors;
pub mod files;
mod loader;
mod logging;
mod manager;

#[cfg(feature = "network")]
mod network;

#[cfg(feature = "network")]
pub use network::{
    middleware::cookies::CookieMiddleware, policy::DocumentPolicy, policy::referrer::ReferrerPolicy,
    request::RequestResult,
};

pub use cache::errors::CacheRead;
pub use cache::memory::{CacheEntry, MemoryCache};
pub use files::Entry;
pub use manager::{Resource, ResourceType};
