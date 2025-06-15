use std::collections::HashMap;

use crate::rules::csp::ContentSecurityPolicy;

/// Represents a response from an HTTP request, including status code, headers, size, and body.
/// The `Response` struct encapsulates the details of an HTTP response, allowing for easy access to the response's metadata and content.
///
/// # Fields
/// * `status` - An unsigned 16-bit integer representing the HTTP status code of the response.
/// * `headers` - A `HashMap` containing the response headers, where keys are header names and values are header values.
/// * `size` - An unsigned integer representing the size of the response body in bytes.
/// * `body` - A string containing the body of the response, which may include HTML, JSON, or other content types.
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub size: usize,
    pub body: String,
}

/// Represents the origin of a web page, including its URL and content security policy (CSP).
/// The `Origin` struct encapsulates the base URL and the CSP rules that apply to the page.
///
/// # Fields
/// * `url` - A string representing the base URL of the page.
/// * `csp` - An instance of `ContentSecurityPolicy` that defines the security policies for the page.
pub struct Origin {
    pub url: String,
    pub csp: ContentSecurityPolicy,
}
