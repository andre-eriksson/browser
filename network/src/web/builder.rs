use http::{HeaderMap, HeaderValue};
use reqwest::Client;
use url::{Origin, Url};

use crate::web::client::WebClient;

pub struct WebClientBuilder {
    pub client: Client,
    pub origin: Origin,
    pub client_headers: HeaderMap<HeaderValue>,
}

impl WebClientBuilder {
    pub fn new(client: Client, origin: Origin) -> Self {
        let client_headers = HeaderMap::new();
        WebClientBuilder {
            client,
            origin,
            client_headers,
        }
    }

    pub fn with_headers(mut self, headers: HeaderMap<HeaderValue>) -> Self {
        self.client_headers = headers;
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
        WebClient::new(self.client, self.origin, self.client_headers)
    }
}
