use std::collections::HashMap;

use crate::data::{Origin, Response};
use crate::rules::csp::ContentSecurityPolicy;

pub struct Page {
    pub client: reqwest::Client,
    pub base_url: String,
    pub origin: Origin,
}

impl Page {
    pub fn new(client: reqwest::Client, base_url: &str) -> Self {
        Page {
            client,
            base_url: base_url.to_string(),
            origin: Origin {
                csp: ContentSecurityPolicy::default(),
            },
        }
    }

    pub async fn fetch(&mut self) -> Result<Response, String> {
        let response = self.client.get(&self.base_url).send().await;

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

        //println!("Page fetched: {} (Status: {})", self.base_url, status);
        //println!("Headers: {:?}", headers);
        //println!("Size: {} bytes", size);

        let origin = Origin {
            csp: headers.get("content-security-policy").map_or_else(
                || ContentSecurityPolicy::default(),
                |csp_header| ContentSecurityPolicy::from_header_string(csp_header),
            ),
        };
        self.origin = origin;

        Ok(Response {
            status,
            headers,
            size,
            body,
        })
    }

    pub async fn get_resource(&self, element: &str, url: &str) -> Result<Response, String> {
        // Use url crate to properly resolve URLs
        let base =
            url::Url::parse(&self.base_url).map_err(|e| format!("Invalid base URL: {}", e))?;

        let resolved_url = base
            .join(url)
            .map_err(|e| format!("Failed to resolve URL '{}': {}", url, e))?;

        let final_url = resolved_url.as_str();
        println!("Fetching resource from: {}", final_url);

        if self.origin.csp.is_blocked(&self.base_url, element, final_url) {
            println!("Content-Security-Policy: {:?}", self.origin.csp);
            return Err(format!(
                "Blocked by Content-Security-Policy: {} for element: {}",
                final_url, element
            ));
        }

        self.fetch_resource(final_url).await
    }

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
