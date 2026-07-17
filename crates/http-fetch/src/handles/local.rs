use async_trait::async_trait;
use bytes::Bytes;
use http::{HeaderMap, StatusCode};
use http_types::{
    body::HttpBody,
    response::{CompleteResponse, HeaderResponse, Response},
};

use crate::{errors::NetworkError, handle::ResponseHandle};

/// A response handle that wraps a complete response and returns it as a cached response.
pub struct LocalHandle {
    head: HeaderResponse,
    body: Bytes,
}

impl LocalHandle {
    pub fn new(response: CompleteResponse) -> Self {
        Self {
            head: response.head,
            body: response.body.0,
        }
    }
}

#[async_trait]
impl ResponseHandle for LocalHandle {
    fn head(&self) -> &HeaderResponse {
        &self.head
    }

    async fn response(self: Box<Self>) -> Result<Response, NetworkError> {
        Ok(Response {
            head: self.head,
            body: HttpBody::Buffered(self.body),
        })
    }
}

impl From<Bytes> for LocalHandle {
    fn from(data: Bytes) -> Self {
        Self {
            head: HeaderResponse::new(StatusCode::OK, HeaderMap::new()),
            body: data,
        }
    }
}

impl From<LocalHandle> for Box<dyn ResponseHandle> {
    fn from(handle: LocalHandle) -> Self {
        Box::new(handle)
    }
}
