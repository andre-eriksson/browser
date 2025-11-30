//! HTTP module for handling HTTP requests and responses.

/// Our HTTP client trait, which all HTTP client implementations should follow.
pub mod client;

/// HTTP request structures and utilities.
pub mod request;

/// HTTP response structures and utilities.
pub mod response;
