use std::time::Instant;

use api::logging::{
    DURATION, EVENT, EVENT_FETCH_CONTENT, EVENT_PAGE_RETRIEVED, STATUS_CODE, TAG_TYPE, URL,
};
use http::{
    HeaderMap, HeaderValue, Method,
    header::{ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, ORIGIN},
};
use reqwest::{Client, Response};
use tracing::{debug, error, info, warn};
use url::{Origin, Url};

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
#[derive(Debug)]
pub struct WebClient {
    pub client: Client,
    pub origin: Origin,
    pub client_header: HeaderMap<HeaderValue>,
    origin_headers: Option<HeaderMap<HeaderValue>>,
}

impl WebClient {
    pub fn new(client: Client, origin: Origin, client_header: HeaderMap<HeaderValue>) -> Self {
        WebClient {
            client,
            origin,
            client_header,
            origin_headers: None,
        }
    }

    pub fn builder(client: Client) -> WebClientBuilder {
        WebClientBuilder {
            client,
            origin: Origin::new_opaque(),
            client_headers: HeaderMap::new(),
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
    async fn setup_client(&mut self, path: &str) -> Result<String, String> {
        let res = self
            .client
            .get(self.origin.unicode_serialization() + path)
            .send()
            .await;

        match res {
            Ok(resp) => {
                if resp.status().is_success() {
                    self.origin_headers = Some(resp.headers().clone());

                    debug!({STATUS_CODE} = ?resp.status());

                    match resp.text().await {
                        Ok(content) => Ok(content),
                        Err(e) => {
                            error!("Failed to read response body: {}", e);
                            return Err(format!("Failed to read response body: {}", e));
                        }
                    }
                } else {
                    warn!({STATUS_CODE} = ?resp.status());
                    Err(format!("{}:{}", STATUS_CODE, resp.status()))
                }
            }
            Err(e) => {
                error!("Failed to send request: {}", e);
                Err(format!("{}", e))
            }
        }
    }

    /// Set up the client using a URL string. This will parse the URL and set the origin accordingly.
    ///
    ///  # Arguments
    /// * `url`: A string slice that holds the URL to set as the origin.
    ///
    ///  # Returns
    ///  A `Result` containing the response body as a `String` if successful, or an error message if the request fails.
    pub async fn setup_client_from_url(&mut self, url: &str) -> Result<String, String> {
        let start_time = Instant::now();
        let parsed_url = Url::parse(url);
        if let Err(e) = parsed_url {
            return Err(format!("Failed to parse URL: {}", e));
        }
        let url = parsed_url.unwrap();

        self.origin = Origin::Tuple(
            url.scheme().to_string(),
            url::Host::Domain(
                url.host_str()
                    .map_or_else(|| String::new(), |s| s.to_string()),
            ),
            url.port_or_known_default().unwrap_or(80),
        );

        let response_result = self
            .setup_client(&url.path())
            .await
            .map_err(|e| format!("{}", e));

        info!(
        {EVENT} = EVENT_PAGE_RETRIEVED,
        {URL} = ?url.as_str(),
        {DURATION} = ?start_time.elapsed(),
        );

        response_result
    }

    /// Fetch content from a specific tag with optional headers and body. E.g. `src` attribute of a `<script>` tag.
    ///
    /// # Arguments
    /// * `tag_name`: The name of the tag to fetch content from (e.g., "script", "link").
    /// * `request_origin`: The origin of the request, which is used to check against Content Security Policy (CSP).
    /// * `http_method`: Optional HTTP method to use for the request (e.g., GET, POST).
    /// * `additional_headers`: Optional headers to include in the request.
    /// * `body`: Optional body content for the request, used for methods like POST.
    ///
    /// # Returns
    /// A `Result` containing the fetched content as a `String` if successful, or an error message if the request fails or is blocked by CSP.
    pub async fn fetch(
        &self,
        tag_name: &str,
        request_origin: &Origin,
        url: &str,
        http_method: Option<Method>,
        additional_headers: Option<HeaderMap<HeaderValue>>,
        _body: Option<String>,
    ) -> Result<Response, String> {
        let start_time = Instant::now();
        let csp_test = handle_csp(
            &self.origin_headers.clone().unwrap_or_default(),
            tag_name,
            request_origin,
        );

        if let Err(e) = csp_test {
            warn!("CSP violation for tag '{}': {}", tag_name, e);
            return Err(format!("CSP violation for tag '{}': {}", tag_name, e));
        }

        // Combine the client headers with the provided headers
        let mut headers = self.client_header.clone();
        if let Some(additional_header) = additional_headers {
            for (key, value) in additional_header.iter() {
                headers.insert(key.clone(), value.clone());
            }
        }

        let http_method = http_method.unwrap_or(Method::GET);

        if !is_simple_request(headers.clone(), http_method.clone())
            && self.origin != request_origin.clone()
        {
            if let Err(e) = self
                .handle_preflight(request_origin, http_method.clone(), headers.clone(), url)
                .await
            {
                warn!("Preflight request failed: {}", e);
                return Err(format!("Preflight request failed: {}", e));
            }
        }

        let res = self
            .client
            .request(http_method, url)
            .headers(headers)
            .build();

        if let Err(e) = res {
            error!("Failed to build request for URI {} | error {}", url, e);
            return Err(format!("Failed to build request: {}", e));
        }

        let response_result = self.client.execute(res.unwrap()).await;

        if let Err(e) = response_result {
            warn!("Failed to execute request: {}", e);
            return Err(format!("Failed to execute request: {}", e));
        }
        let response = response_result.unwrap();
        if !response.status().is_success() {
            warn!({EVENT} = {EVENT_FETCH_CONTENT}, {TAG_TYPE} = ?tag_name, {URL} = ?url, {STATUS_CODE} = ?response.status(), {DURATION} = ?start_time.elapsed());
        } else {
            debug!({EVENT} = {EVENT_FETCH_CONTENT}, {TAG_TYPE} = ?tag_name, {URL} = ?url, {STATUS_CODE} = ?response.status(), {DURATION} = ?start_time.elapsed());
        }

        return Ok(response);
    }

    /// Fetch content from the origin with optional headers and body.
    ///
    /// # Arguments
    /// * `tag_name`: The name of the tag to fetch content from (e.g., "script", "link").
    /// * `path`: The path to fetch from the origin.
    /// * `http_method`: Optional HTTP method to use for the request (e.g., GET, POST).
    /// * `additional_headers`: Optional headers to include in the request.
    /// * `body`: Optional body content for the request, used for methods like POST.
    ///
    /// # Returns
    /// A `Result` containing the fetched content as a `String` if successful, or an error message if the request fails or is blocked by CSP.
    pub async fn fetch_from_origin(
        &self,
        tag_name: &str,
        path: &str,
        http_method: Option<Method>,
        additional_headers: Option<HeaderMap<HeaderValue>>,
        body: Option<String>,
    ) -> Result<Response, String> {
        self.fetch(
            tag_name,
            &self.origin,
            path,
            http_method,
            additional_headers,
            body,
        )
        .await
    }
}
