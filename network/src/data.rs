/// Represents a response from an HTTP request, including status code, headers, size, and body.
///
/// The `Response` struct encapsulates the details of an HTTP response, allowing for easy access to the response's metadata and content.
pub struct Response {
    /// An unsigned integer representing the size of the response body in bytes.
    pub size: usize,
    /// A string containing the body of the response, which may include HTML, JSON, or other content types.
    pub body: String,
}
