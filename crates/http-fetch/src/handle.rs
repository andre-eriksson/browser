use async_trait::async_trait;

use http_types::response::{HeaderResponse, Response};

use crate::errors::NetworkError;

/// A handle to a response, providing access to the response head and body.
#[async_trait]
pub trait ResponseHandle: Send + Sync {
    /// Returns the head of the response, without consuming the body.
    fn head(&self) -> &HeaderResponse;

    /// Consumes and returns the full response, buffering if necessary.
    async fn response(self: Box<Self>) -> Result<Response, NetworkError>;
}
