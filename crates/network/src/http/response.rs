use http::{HeaderMap, StatusCode};

/// Represents the first part of an HTTP response, containing headers and status code.
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/Response>
pub struct HeaderResponse {
    /// The status code of the response.
    ///
    /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Status>
    pub status_code: StatusCode,

    /// The headers of the response.
    pub headers: HeaderMap,
}

/// Represents a complete HTTP response, including headers, status code, and body.
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/Response>
#[derive(Clone, Debug)]
pub struct Response {
    /// The status code of the response.
    ///
    /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Status>
    pub status_code: StatusCode,

    /// The headers of the response.
    pub headers: HeaderMap,

    /// The body of the response.
    pub body: Option<Vec<u8>>,
}

impl HeaderResponse {
    /// Creates a new HTTP response.
    ///
    /// Useful for testing and constructing responses manually.
    pub fn new(status_code: StatusCode, headers: HeaderMap) -> Self {
        HeaderResponse {
            status_code,
            headers,
        }
    }
}
