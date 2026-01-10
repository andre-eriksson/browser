//! Middleware modules for handling various aspects of network sessions.

/// Cookie management middleware for sessions.
pub mod cookies;

/// CORS (Cross-Origin Resource Sharing) middleware for sessions.
///
/// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS>
pub mod cors;

/// Referrer handling middleware for sessions.
///
/// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Referrer-Policy>
pub mod referrer;

/// Simple request checking middleware for sessions.
///
/// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS#simple_requests>
pub mod simple;
