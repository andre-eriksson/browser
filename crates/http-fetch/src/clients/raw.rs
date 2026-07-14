use async_trait::async_trait;
use bytes::Bytes;
use http::{HeaderMap, StatusCode};
use http_types::{
    body::HttpBody,
    response::{HeaderResponse, Response},
};

use crate::{client::ResponseHandle, errors::NetworkError};

/// A simple HTTP client that returns a raw response with a given a pre-determined body.
pub struct RawClient {
    raw_header: HeaderResponse,
    data: Bytes,
}

impl RawClient {
    pub fn wrap_handle(data: Bytes) -> Box<dyn ResponseHandle> {
        Box::new(Self {
            raw_header: HeaderResponse::new(StatusCode::OK, HeaderMap::new()),
            data,
        })
    }
}

#[async_trait]
impl ResponseHandle for RawClient {
    fn head(&self) -> &HeaderResponse {
        &self.raw_header
    }

    async fn response(self: Box<Self>) -> Result<Response, NetworkError> {
        let response = Response {
            head: self.raw_header,
            body: HttpBody::Buffered(self.data),
        };

        Ok(response)
    }
}
