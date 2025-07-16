use http::{HeaderMap, HeaderValue};
use reqwest::{Client, redirect::Policy};
use tracing::error;
use url::{Origin, Url};

use crate::web::client::WebClient;

/// A builder for creating a `WebClient` with customizable settings.
///
/// # Fields
/// * `max_redirects` - The maximum number of redirects to follow, defaults to 10.
/// * `user_agent` - The User-Agent header to be used, defaults to a specific string.
/// * `origin` - The origin of the client, typically set to `Origin::new_opaque()` initially.
/// * `client_headers` - A `HeaderMap` containing custom headers for the client.
/// * `client` - An optional `reqwest::Client` instance if you want to bypass the builder and use a custom client directly.
pub struct WebClientBuilder {
    max_redirects: usize,
    user_agent: String,
    pub origin: Origin,
    pub client_headers: HeaderMap<HeaderValue>,
    pub client: Option<Client>,
}

impl WebClientBuilder {
    /// Creates a new `WebClientBuilder` with default values.
    ///
    /// # Arguments
    /// * `origin` - The origin to be used by the WebClient, typically set to `Origin::new_opaque()` for initialization,
    ///   should be set later with `WebClient::setup_client_from_url`, when you have made the first request.
    pub fn new(origin: Origin) -> Self {
        let client_headers = HeaderMap::new();
        WebClientBuilder {
            max_redirects: 10,
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) egui/0.31.0 (KHTML, like Gecko) Rust/1.87.0 MiniBrowser/0.1.0".to_string(),
            origin,
            client_headers,
            client: None,
        }
    }

    /// Sets the HTTP client to be used by the WebClient. Is mostly used for testing purposes.
    ///
    /// # Arguments
    /// * `client` - An instance of `reqwest::Client` to be used for HTTP requests.
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Sets the maximum number of redirects for the HTTP client.
    ///
    /// # Arguments
    /// * `max_redirects` - The maximum number of redirects to follow.
    pub fn max_redirects(mut self, max_redirects: usize) -> Self {
        self.max_redirects = max_redirects;
        self
    }

    /// Sets the `User-Agent` header.
    ///
    /// # Arguments
    /// * `user_agent` - A string slice that holds the value for the User-Agent header.
    ///
    /// # See Also
    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/User-Agent
    pub fn with_user_agent(mut self, user_agent: &str) -> Self {
        self.user_agent = user_agent.to_string();
        self
    }

    /// Sets custom headers for the client.
    ///
    /// # Arguments
    /// * `headers` - A `HeaderMap` containing the headers to be set.
    pub fn with_headers(mut self, headers: HeaderMap<HeaderValue>) -> Self {
        self.client_headers = headers;
        self
    }

    /// Sets the `Accept` header.
    ///
    /// # Arguments
    /// * `accept` - A string slice that holds the value for the Accept header,
    ///
    /// # See Also
    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Accept
    pub fn with_accept(mut self, accept: &str) -> Self {
        self.client_headers.insert(
            http::header::ACCEPT,
            HeaderValue::from_str(accept).expect("Invalid Accept header"),
        );
        self
    }

    /// Sets the `Accept-Language` header.
    ///
    /// # Arguments
    /// * `accept_language` - A string slice that holds the value for the Accept-Language header, typically "en-US,en;q=0.9".
    ///
    /// # See Also
    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Accept-Language
    pub fn with_accept_language(mut self, accept_language: &str) -> Self {
        self.client_headers.insert(
            http::header::ACCEPT_LANGUAGE,
            HeaderValue::from_str(accept_language).expect("Invalid Accept-Language header"),
        );
        self
    }

    /// Sets the `Accept-Encoding` header.
    ///
    /// # Arguments
    /// * `accept_encoding` - A string slice that holds the value for the Accept-Encoding header, typically "gzip, deflate, br".
    ///
    /// # See Also
    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Accept-Encoding
    pub fn with_accept_encoding(mut self, accept_encoding: &str) -> Self {
        self.client_headers.insert(
            http::header::ACCEPT_ENCODING,
            HeaderValue::from_str(accept_encoding).expect("Invalid Accept-Encoding header"),
        );
        self
    }

    /// Sets the `Connection` header.
    ///
    /// # Arguments
    /// * `connection` - A string slice that holds the value for the Connection header, usually "keep-alive" or "close".
    ///
    /// # See Also
    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Connection
    pub fn with_connection(mut self, connection: &str) -> Self {
        self.client_headers.insert(
            http::header::CONNECTION,
            HeaderValue::from_str(connection).expect("Invalid Connection header"),
        );
        self
    }

    /// Sets the `Upgrade-Insecure-Requests` header.
    ///
    /// # Arguments
    /// * `value` - A '1' is the only valid value for this header.
    ///
    /// # See Also
    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Upgrade-Insecure-Requests
    pub fn with_upgrade_insecure_requests(mut self, value: &str) -> Self {
        self.client_headers.insert(
            http::header::UPGRADE_INSECURE_REQUESTS,
            HeaderValue::from_str(value).expect("Invalid Upgrade-Insecure-Requests header"),
        );
        self
    }

    /// Set the origin based on a URL.
    ///
    /// # Arguments
    /// * `url` - A string slice that holds the URL to set as the origin.
    ///
    /// # Returns
    /// A new `WebClientBuilder` instance with the origin set to the parsed URL.
    pub fn with_url(mut self, url: &str) -> Self {
        let url = Url::parse(url).expect("Invalid URL");
        self.origin = Origin::Tuple(
            url.scheme().to_owned(),
            url.host().unwrap().to_owned(),
            url.port_or_known_default().unwrap_or(0),
        );
        self
    }

    /// Finalizes the builder and returns a `WebClient` instance.
    pub fn build(self) -> WebClient {
        if self.client.is_some() {
            return WebClient::new(self.client.unwrap(), self.origin, self.client_headers);
        }

        let client = Client::builder()
            .user_agent(self.user_agent)
            .redirect(Policy::limited(self.max_redirects))
            .build();

        if let Err(e) = client {
            error!("Failed to create HTTP client: {}", e);
            panic!("Failed to create HTTP client: {}", e);
        }
        let client = client.unwrap();

        WebClient::new(client, self.origin, self.client_headers)
    }
}
