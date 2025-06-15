use std::collections::HashMap;

use crate::data::{Origin, Response};
use crate::rules::csp::ContentSecurityPolicy;

/// The first function to call to fetch a page.
pub async fn fetch_page(url: &str) -> Result<(Response, Origin), ()> {
    let response = reqwest::get(url).await;

    match response {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let headers: HashMap<String, String> = resp
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();
            let size = resp.content_length().unwrap_or(0) as usize;
            let body = resp.text().await.unwrap_or_default();

            let csp_header = headers
                .get("content-security-policy")
                .cloned()
                .unwrap_or_default();

            let csp = ContentSecurityPolicy::from_header_string(&csp_header);

            Ok((
                Response {
                    status,
                    headers,
                    size,
                    body,
                },
                Origin { csp },
            ))
        }
        Err(e) => {
            println!("Error: {}", e);
            return Err(());
        }
    }
}

pub async fn get_resource(page: Origin, element: &str, url: &str) -> Result<Response, String> {
    if page.csp.is_blocked(element, url) {
        return Err(format!(
            "Blocked by Content-Security-Policy: {} for element: {}",
            url, element
        ));
    }

    let response = reqwest::get(url).await;

    match response {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let headers: HashMap<String, String> = resp
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();
            let size = resp.content_length().unwrap_or(0) as usize;
            let body = resp.text().await.unwrap_or_default();

            println!("Resource fetched: {} (Status: {})", url, status);
            println!("Headers: {:?}", headers);
            println!("Size: {} bytes", size);

            Ok(Response {
                status,
                headers,
                size,
                body,
            })
        }
        Err(e) => {
            return Err(format!("Failed to fetch resource {}: {}", url, e));
        }
    }
}
