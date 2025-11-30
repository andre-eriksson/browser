use async_trait::async_trait;

use crate::http::{request::Request, response::Response};

/// An asynchronous HTTP client trait.
///
/// This trait defines the interface for sending HTTP requests and receiving responses.
#[async_trait]
pub trait HttpClient {
    /// Sends an HTTP request and returns the response.
    ///
    /// # Arguments
    /// * `request` - The HTTP request to be sent.
    ///
    /// # Returns
    /// * `Result<Response, Box<dyn std::error::Error>>` - The HTTP response or an error.
    async fn send(&self, request: Request) -> Result<Response, Box<dyn std::error::Error>>;
}
