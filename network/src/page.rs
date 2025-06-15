use std::collections::HashMap;

use crate::data::{Origin, Response};
use crate::rules::csp::ContentSecurityPolicy;

/// Represents a web page that can be fetched and parsed, including its content security policy (CSP).
/// The `Page` struct encapsulates the HTTP client, base URL, and origin information,
///   allowing for fetching the page and its resources while respecting the CSP rules.
///
/// # Fields
/// * `client` - An instance of `reqwest::Client` used for making HTTP requests.
/// * `origin` - An `Origin` struct that contains the base URL and the content security policy for the page.
pub struct Page {
    pub client: reqwest::Client,
    pub origin: Origin,
}

impl Page {
    /// Creates a new `Page` instance with the given HTTP client and base URL. To start fetching resources, you must first call the `fetch` method,
    ///   this means you can run functions before fetching the page, if needed.
    ///
    /// # Arguments
    /// * `client` - An instance of `reqwest::Client` used for making HTTP requests.
    /// * `origin_url` - A string slice representing the base URL of the page to be fetched.
    ///
    /// # Returns
    /// A new `Page` instance initialized with the provided client and base URL.
    pub fn new(client: reqwest::Client, origin_url: &str) -> Self {
        Page {
            client,
            origin: Origin {
                url: origin_url.to_string(),
                csp: ContentSecurityPolicy::default(),
            },
        }
    }

    /// Fetch the page at the base URL and return a `Response` containing the status, headers, size, and body.
    /// Updates the `origin` field with the Content Security Policy from the response headers.
    ///    
    /// # Returns
    /// A `Result` containing the `Response` if successful, or an error message if it fails.
    pub async fn fetch(&mut self) -> Result<Response, String> {
        let response = self.client.get(&self.origin.url).send().await;

        if let Err(e) = response {
            return Err(format!("Failed to fetch page: {}", e));
        }

        let resp = response.unwrap();
        let status = resp.status().as_u16();
        let headers: HashMap<String, String> = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let size = resp.content_length().unwrap_or(0) as usize;
        let body = resp.text().await.unwrap_or_default();

        self.origin.csp = headers.get("content-security-policy").map_or_else(
            || ContentSecurityPolicy::default(),
            |csp_header| ContentSecurityPolicy::from_header_string(csp_header),
        );

        Ok(Response {
            status,
            headers,
            size,
            body,
        })
    }

    /// Fetch a resource from the page, resolving relative URLs against the base URL.
    ///
    /// # Arguments
    /// * `element` - The type of element being fetched (e.g., "script", "link", "img").
    /// * `url` - The URL of the resource to fetch, which can be relative or absolute.
    ///
    /// # Returns
    /// A `Result` containing the `Response` if successful, or an error message if it fails.
    pub async fn get_resource(&self, element: &str, url: &str) -> Result<Response, String> {
        // Use url crate to properly resolve URLs
        let base =
            url::Url::parse(&self.origin.url).map_err(|e| format!("Invalid base URL: {}", e))?;

        let resolved_url = base
            .join(url)
            .map_err(|e| format!("Failed to resolve URL '{}': {}", url, e))?;

        let final_url = resolved_url.as_str();
        println!("Fetching resource from: {}", final_url);

        if self
            .origin
            .csp
            .is_blocked(&self.origin.url, element, final_url)
        {
            println!("Content-Security-Policy: {:?}", self.origin.csp);
            return Err(format!(
                "Blocked by Content-Security-Policy: {} for element: {}",
                final_url, element
            ));
        }

        self.fetch_resource(final_url).await
    }

    /// A utility function to fetch a resource from a given URL.
    /// This function is used internally to perform the actual HTTP request and return the response.
    ///
    /// # Arguments
    /// * `url` - The URL of the resource to fetch.
    ///
    /// # Returns
    /// A `Result` containing the `Response` if successful, or an error message if it fails.
    async fn fetch_resource(&self, url: &str) -> Result<Response, String> {
        let response = self.client.get(url).send().await;

        if let Err(e) = response {
            return Err(format!("Failed to fetch resource: {}", e));
        }

        let resp = response.unwrap();
        let status = resp.status().as_u16();
        let headers: HashMap<String, String> = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let size = resp.content_length().unwrap_or(0) as usize;
        let body = resp.text().await.unwrap_or_default();

        Ok(Response {
            status,
            headers,
            size,
            body,
        })
    }
}
