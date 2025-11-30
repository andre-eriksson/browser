use http::{HeaderMap, StatusCode};

/// Represents an HTTP response.
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/Response>
pub struct Response {
    /// The status code of the response.
    ///
    /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Status>
    pub status_code: StatusCode,

    /// The headers of the response.
    pub headers: HeaderMap,

    /// The body of the response.
    pub body: Vec<u8>,
}

impl Response {
    /// Creates a new HTTP response.
    ///
    /// Useful for testing and constructing responses manually.
    pub fn new(status_code: StatusCode, headers: HeaderMap, body: Vec<u8>) -> Self {
        Response {
            status_code,
            headers,
            body,
        }
    }
}
