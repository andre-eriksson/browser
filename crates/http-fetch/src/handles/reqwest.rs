use async_trait::async_trait;
use http_types::{
    body::HttpBody,
    response::{HeaderResponse, Response},
};

use crate::{errors::NetworkError, handle::ResponseHandle};

#[derive(Debug)]
pub struct ReqwestHandle {
    inner: reqwest::Response,
    head: HeaderResponse,
}

impl ReqwestHandle {
    pub fn new(inner: reqwest::Response, head: HeaderResponse) -> Self {
        Self { inner, head }
    }
}

#[async_trait]
impl ResponseHandle for ReqwestHandle {
    fn head(&self) -> &HeaderResponse {
        &self.head
    }

    async fn response(self: Box<Self>) -> Result<Response, NetworkError> {
        let status_code = self.head.status_code;
        let headers = self.head.headers;

        let body_bytes = self.inner.bytes().await;

        let body_bytes = match body_bytes {
            Ok(bytes) => bytes.to_vec(),
            Err(err) => {
                return match err {
                    _ if err.is_timeout() => Err(NetworkError::Timeout),
                    _ if err.is_redirect() => Err(NetworkError::MaxRedirectsExceeded),
                    _ if err.is_connect() => Err(NetworkError::ConnectionRefused),
                    e => Err(NetworkError::InvalidRequest(e.to_string())),
                };
            }
        };

        Ok(Response {
            head: HeaderResponse {
                headers,
                status_code,
            },
            body: HttpBody::Buffered(body_bytes.into()),
        })
    }
}
