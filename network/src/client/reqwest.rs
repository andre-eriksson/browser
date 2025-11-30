use async_trait::async_trait;

use crate::http::{client::HttpClient, request::Request, response::Response};

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

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn send(&self, request: Request) -> Result<Response, Box<dyn std::error::Error>> {
        let mut req = self.client.request(request.method, request.url);

        for (key, value) in request.headers.iter() {
            req = req.header(key, value);
        }

        if let Some(body) = request.body {
            req = req.body(body);
        }

        let response = req.send().await?;

        let status_code = response.status();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let body = response.bytes().await?.to_vec();

        Ok(Response {
            status_code,
            headers,
            body,
        })
    }
}
