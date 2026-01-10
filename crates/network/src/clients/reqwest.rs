use async_trait::async_trait;
use errors::network::NetworkError;
use telemetry::keys::STATUS_CODE;
use tracing::{debug, error, instrument, trace};

use crate::http::{
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
            Err(e) => return Err(NetworkError::RequestFailed(e.to_string())),
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
    #[instrument(skip(self, request), fields(method = %request.method, url = %request.url))]
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
            Err(e) => {
                error!("Request failed: {}", e);
                return Err(NetworkError::RequestFailed(e.to_string()));
            }
        };

        let status_code = response.status();

        debug!({ STATUS_CODE } = format!("{}", status_code));

        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| {
                let key = k.clone();
                let value = v.clone();

                trace!(
                    "Response Header: {}: {:?}",
                    key.as_str(),
                    value.to_str().unwrap_or("<invalid utf8>")
                );

                (key, value)
            })
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
