use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use http_types::{
    body::{BodyStream, HttpBody},
    request::RequestContext,
    response::{HeaderResponse, Response},
};

use crate::errors::NetworkError;

#[async_trait]
pub trait ResponseHandle: Send + Sync {
    fn head(&self) -> &HeaderResponse;
    /// Consumes and returns the full response, buffering if necessary.
    async fn response(self: Box<Self>) -> Result<Response, NetworkError>;

    /// Consumes and returns the body as a stream. Default buffers eagerly
    /// via `response()`; real streaming clients can override this later
    /// without changing the trait surface.
    async fn body_stream(self: Box<Self>) -> Result<BodyStream, NetworkError> {
        let resp = self.response().await?;
        Ok(resp.body.into_stream())
    }
}

/// An asynchronous HTTP client trait.
///
/// This trait defines the interface for sending HTTP requests and receiving responses.
#[async_trait]
pub trait HttpClient: Send + Sync + Debug {
    /// Sends an HTTP request and returns the response.
    ///
    /// # Arguments
    /// * `request` - The HTTP request to be sent.
    ///
    /// # Returns
    /// * `Result<Response, Box<dyn std::error::Error>>` - The HTTP response or an error.
    async fn send(&self, context: Arc<RequestContext>, body: HttpBody)
    -> Result<Box<dyn ResponseHandle>, NetworkError>;

    fn box_clone(&self) -> Box<dyn HttpClient>;
}

impl Clone for Box<dyn HttpClient> {
    fn clone(&self) -> Self {
        self.as_ref().box_clone()
    }
}
