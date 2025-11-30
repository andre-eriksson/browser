//! Network module for handling HTTP clients and sessions.

/// This module provides functionalities for creating HTTP clients/backends.
/// Handles the actual network requests and responses. But all implementations
/// should follow the `http::HttpClient` trait.
pub mod client;

/// This module contains HTTP-related structures and traits.
pub mod http;

/// This module manages sessions (tabs), which encapsulate stateful interactions
/// with web servers, including cookies, and other session-specific data.
pub mod session;
