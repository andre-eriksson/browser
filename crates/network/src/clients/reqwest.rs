use crate::errors::NetworkError;
use async_trait::async_trait;

use crate::{
    client::{HttpClient, ResponseHandle},
    request::Request,
    response::{HeaderResponse, Response},
};

/// An HTTP client implementation using the `reqwest` library.
#[derive(Default)]
pub struct ReqwestClient {
    /// The underlying reqwest client.
    client: reqwest::Client,
}

impl ReqwestClient {
    pub fn new() -> Self {
        ReqwestClient {
            client: reqwest::Client::new(),
        }
    }
}

pub struct ReqwestHandle {
    inner: reqwest::Response,
    metadata: HeaderResponse,
}

#[async_trait]
impl ResponseHandle for ReqwestHandle {
    fn metadata(&self) -> &HeaderResponse {
        &self.metadata
    }

    async fn body(self: Box<Self>) -> Result<Response, NetworkError> {
        let status_code = self.metadata.status_code;
        let headers = self.metadata.headers;

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
            status_code,
            headers,
            body: Some(body_bytes),
        })
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn send(&self, request: Request) -> Result<Box<dyn ResponseHandle>, NetworkError> {
        let mut req = self.client.request(request.method, request.url);

        for (key, value) in request.headers.iter() {
            req = req.header(key, value);
        }

        if let Some(body) = request.body {
            req = req.body(body);
        }

        let response = req.send().await;

        let response = match response {
            Ok(resp) => resp,
            Err(err) => {
                return match err {
                    _ if err.is_timeout() => Err(NetworkError::Timeout),
                    _ if err.is_redirect() => Err(NetworkError::MaxRedirectsExceeded),
                    _ if err.is_connect() => Err(NetworkError::ConnectionRefused),
                    e => Err(NetworkError::InvalidRequest(e.to_string())),
                };
            }
        };

        let status_code = response.status();

        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let metadata = HeaderResponse {
            status_code,
            headers,
        };

        Ok(Box::new(ReqwestHandle {
            inner: response,
            metadata,
        }))
    }

    fn box_clone(&self) -> Box<dyn HttpClient> {
        Box::new(ReqwestClient {
            client: self.client.clone(),
        })
    }
}
