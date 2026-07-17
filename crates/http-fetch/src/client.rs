use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;

use http_types::{body::HttpBody, request::RequestContext};

use crate::{errors::NetworkError, handle::ResponseHandle};

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
