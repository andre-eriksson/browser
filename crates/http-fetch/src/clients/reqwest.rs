use std::sync::Arc;

use async_trait::async_trait;

use http::Method;
use http_types::{body::HttpBody, request::RequestContext, response::HeaderResponse};
use reqwest::redirect::Policy;

use crate::{client::HttpClient, errors::NetworkError, handle::ResponseHandle, handles::ReqwestHandle};

/// An HTTP client implementation using the `reqwest` library.
#[derive(Debug, Default)]
pub struct ReqwestClient {
    /// The underlying reqwest client.
    client: reqwest::Client,
    /// The preflight client used for OPTIONS requests.
    preflight_client: reqwest::Client,
}

impl ReqwestClient {
    /// Creates a new instance of `ReqwestClient` with default settings.
    ///
    /// # Returns
    /// A new `ReqwestClient` instance ready to send HTTP requests.
    ///
    /// # Panics
    /// Panics if the reqwest client fails to build, which is unlikely under normal circumstances.
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .no_brotli()
                .no_deflate()
                .no_gzip()
                .no_zstd()
                .http2_max_header_list_size(u16::MAX as u32)
                .build()
                .unwrap(),
            preflight_client: reqwest::Client::builder()
                .redirect(Policy::none())
                .no_brotli()
                .no_deflate()
                .no_gzip()
                .no_zstd()
                .http2_max_header_list_size(u16::MAX as u32)
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn send(
        &self,
        context: Arc<RequestContext>,
        body: HttpBody,
    ) -> Result<Box<dyn ResponseHandle>, NetworkError> {
        let mut request = if context.method == Method::OPTIONS {
            self.preflight_client
                .request(context.method.clone(), context.url.clone())
        } else {
            self.client
                .request(context.method.clone(), context.url.clone())
        };

        for (key, value) in &context.headers {
            request = request.header(key, value);
        }

        match body {
            HttpBody::Empty => {}
            HttpBody::Buffered(bytes) => request = request.body(bytes),
            HttpBody::Streaming(_) => unimplemented!("Stream body requests aren't supported in the reqwest client"),
        }

        let response = match request.send().await {
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

        Ok(Box::new(ReqwestHandle::new(response, metadata)))
    }

    fn box_clone(&self) -> Box<dyn HttpClient> {
        Box::new(Self {
            client: self.client.clone(),
            preflight_client: self.preflight_client.clone(),
        })
    }
}
