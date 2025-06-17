use http::{HeaderMap, HeaderValue, Method};
use reqwest::Client;
use url::Origin;

use crate::{headers::csp::handle_csp, web::builder::WebClientBuilder};

pub struct WebClient {
    pub client: Client,
    pub origin: Origin,
    pub headers: Option<HeaderMap<HeaderValue>>,
}

impl WebClient {
    pub fn new(client: Client, origin: Origin) -> Self {
        WebClient {
            client,
            origin,
            headers: None,
        }
    }

    pub fn builder(client: Client) -> WebClientBuilder {
        WebClientBuilder {
            client,
            origin: Origin::new_opaque(),
            headers: HeaderMap::new(),
        }
    }

    /// Initialize the origin by sending a GET request to the specified URL. Will set the headers
    /// for the client if the request is successful.
    ///
    /// # Returns
    /// A `Result` containing the response body as a `String` if successful, or an error message if the request fails.
    pub async fn setup_client(&mut self) -> Result<String, String> {
        let res = self
            .client
            .get(self.origin.unicode_serialization())
            .send()
            .await;

        match res {
            Ok(resp) => {
                if resp.status().is_success() {
                    self.headers = Some(resp.headers().clone());

                    match resp.text().await {
                        Ok(content) => Ok(content),
                        Err(e) => {
                            return Err(format!("Failed to read response body: {}", e));
                        }
                    }
                } else {
                    Err(format!("Request failed with status: {}", resp.status()))
                }
            }
            Err(e) => Err(format!("Failed to execute request: {}", e)),
        }
    }

    /// Fetch content from a specific tag with optional headers and body. E.g. `src` attribute of a `<script>` tag.
    ///
    /// # Arguments
    /// * `tag_name`: The name of the tag to fetch content from (e.g., "script", "link").
    /// * `request_origin`: The origin of the request, which is used to check against Content Security Policy (CSP).
    /// * `http_method`: Optional HTTP method to use for the request (e.g., GET, POST).
    /// * `headers`: Optional headers to include in the request.
    /// * `body`: Optional body content for the request, used for methods like POST.
    ///
    /// # Returns
    /// A `Result` containing the fetched content as a `String` if successful, or an error message if the request fails or is blocked by CSP.
    pub async fn fetch(
        &self,
        tag_name: &str,
        request_origin: &Origin,
        path: &str,
        http_method: Option<Method>,
        headers: Option<HeaderMap<HeaderValue>>,
        body: Option<String>,
    ) -> Result<String, String> {
        if let Some(csp_violation) = handle_csp(
            &self.headers.as_ref().unwrap_or(&HeaderMap::new()),
            tag_name,
            request_origin,
        ) {
            return csp_violation;
        }

        let res = self
            .client
            .request(
                http_method.unwrap_or(Method::GET),
                request_origin.unicode_serialization() + path,
            )
            .headers(headers.unwrap_or_else(|| HeaderMap::new()))
            .body(body.unwrap_or(String::new()))
            .build();

        if let Err(e) = res {
            return Err(format!("Failed to build request: {}", e));
        }

        let response = self.client.execute(res.unwrap()).await;
        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let text = resp.text().await;
                    match text {
                        Ok(content) => Ok(content),
                        Err(e) => Err(format!("Failed to read response body: {}", e)),
                    }
                } else {
                    Err(format!("Request failed with status: {}", resp.status()))
                }
            }
            Err(e) => Err(format!("Failed to execute request: {}", e)),
        }
    }

    /// Fetch content from the origin with optional headers and body.
    ///
    /// # Arguments
    /// * `tag_name`: The name of the tag to fetch content from (e.g., "script", "link").
    /// * `path`: The path to fetch from the origin.
    /// * `http_method`: Optional HTTP method to use for the request (e.g., GET, POST).
    /// * `headers`: Optional headers to include in the request.
    /// * `body`: Optional body content for the request, used for methods like POST.
    ///
    /// # Returns
    /// A `Result` containing the fetched content as a `String` if successful, or an error message if the request fails or is blocked by CSP.
    pub async fn fetch_from_origin(
        &self,
        tag_name: &str,
        path: &str,
        http_method: Option<Method>,
        headers: Option<HeaderMap<HeaderValue>>,
        body: Option<String>,
    ) -> Result<String, String> {
        self.fetch(tag_name, &self.origin, path, http_method, headers, body)
            .await
    }
}
