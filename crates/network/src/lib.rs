//! Network module for handling HTTP clients and sessions.

/// This module contains various HTTP client implementations.
pub mod clients;

/// This module contains HTTP-related structures and traits.
pub mod http;

/// This module manages sessions (tabs), which encapsulate stateful interactions
/// with web servers, including cookies, and other session-specific data.
pub mod session;
