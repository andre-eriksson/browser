mod cache;
pub mod embeded;
pub mod errors;
pub mod files;
mod loader;
mod manager;

#[cfg(feature = "network")]
mod network;

#[cfg(feature = "network")]
pub use network::{
    middleware::cookies::CookieMiddleware, policy::DocumentPolicy, request::RequestResult,
};

pub use cache::{Cache, CacheEntry};
pub use manager::{Resource, ResourceType};
