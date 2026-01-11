use async_trait::async_trait;
use errors::network::RequestError;

use crate::http::{
    request::Request,
    response::{HeaderResponse, Response},
};

#[async_trait]
pub trait ResponseHandle: Send + Sync {
    fn metadata(&self) -> &HeaderResponse;
    async fn body(self: Box<Self>) -> Result<Response, RequestError>;
}

/// An asynchronous HTTP client trait.
///
/// This trait defines the interface for sending HTTP requests and receiving responses.
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Sends an HTTP request and returns the response.
    ///
    /// # Arguments
    /// * `request` - The HTTP request to be sent.
    ///
    /// # Returns
    /// * `Result<Response, Box<dyn std::error::Error>>` - The HTTP response or an error.
    async fn send(&self, request: Request) -> Result<Box<dyn ResponseHandle>, RequestError>;

    fn box_clone(&self) -> Box<dyn HttpClient>;
}

impl Clone for Box<dyn HttpClient> {
    fn clone(&self) -> Box<dyn HttpClient> {
        self.as_ref().box_clone()
    }
}
