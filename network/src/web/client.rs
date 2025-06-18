use http::{
    HeaderMap, HeaderValue, Method,
    header::{ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, ORIGIN},
};
use reqwest::Client;
use url::Origin;

use crate::{
    rules::{cors::validate_cors_preflight, csp::handle_csp, simple::is_simple_request},
    web::builder::WebClientBuilder,
};

/// A web client for making HTTP requests with CORS and CSP handling.
/// This client is designed to work with a specific origin and can handle preflight requests for CORS.
/// It also checks Content Security Policy (CSP) rules before making requests.
///
/// # Fields
/// * `client`: The underlying HTTP client used for making requests.
/// * `origin`: The origin of the web client, used for CORS and CSP checks.
/// * `headers`: Base headers from the origin when calling `setup_client`, to get the CSP rules among others.
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

    async fn handle_preflight(
        &self,
        request_origin: &Origin,
        method: Method,
        headers: HeaderMap<HeaderValue>,
        path: &str,
    ) -> Result<(), String> {
        let res = self
            .client
            .request(
                Method::OPTIONS,
                request_origin.unicode_serialization() + path,
            )
            .header(ORIGIN, self.origin.unicode_serialization())
            .header(ACCESS_CONTROL_REQUEST_METHOD, method.as_str())
            .header(
                ACCESS_CONTROL_REQUEST_HEADERS,
                headers
                    .iter()
                    .map(|(k, _)| k.as_str())
                    .collect::<Vec<&str>>()
                    .join(", "),
            )
            .build();

        if let Err(e) = res {
            return Err(format!("Failed to build preflight request: {}", e));
        }

        let response = self.client.execute(res.unwrap()).await;

        if let Err(e) = response {
            return Err(format!("Failed to execute preflight request: {}", e));
        }

        let resp = response.unwrap();

        if !resp.status().is_success() {
            return Err(format!(
                "Preflight request failed with status: {}",
                resp.status()
            ));
        }

        return validate_cors_preflight(&resp.headers(), request_origin, &method, &headers);
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
        let csp_test = handle_csp(
            &self.headers.clone().unwrap_or_default(),
            tag_name,
            request_origin,
        );

        if let Err(e) = csp_test {
            return Err(format!("CSP violation: {}", e));
        }

        let headers = headers
            .or_else(|| self.headers.clone())
            .unwrap_or_else(HeaderMap::new);

        let http_method = http_method.unwrap_or(Method::GET);

        if !is_simple_request(headers.clone(), http_method.clone())
            && self.origin != request_origin.clone()
        {
            if let Err(e) = self
                .handle_preflight(request_origin, http_method.clone(), headers.clone(), path)
                .await
            {
                return Err(format!("Preflight request failed: {}", e));
            }
        }

        let res = self
            .client
            .request(http_method, request_origin.unicode_serialization() + path)
            .headers(headers)
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
