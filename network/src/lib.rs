/// The data module contains all the types and structures used for representing network data.
pub mod data;

/// The rules module contains all the logic for validating and applying various web standards like CORS and CSP.
pub mod rules;

/// The utils module contains various utility functions and types used throughout the network layer.
pub mod util;

/// The web module handles all web-related functionality like using WebClient to handle web page requests and responses
pub mod web;

#[cfg(test)]
mod tests {
    use http::{
        HeaderMap, HeaderValue, Method,
        header::{
            ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS,
            ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_SECURITY_POLICY,
        },
    };
    use reqwest::Client;

    use crate::{
        rules::{cors::validate_cors_preflight, csp::handle_csp},
        web::client::WebClient,
    };

    #[tokio::test]
    async fn test_connection() {
        let client = Client::builder()
        .user_agent(format!(
            "browser-{}/{}-dev (testing; Rust 1.28.2; reqwest 0.12.18) andreeriksson444@gmail.com",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        ))
        .build()
        .expect("Failed to build HTTP client");

        let mut web_client = WebClient::builder().with_client(client).build();

        let result = web_client
            .setup_client_from_url("https://www.example.com")
            .await;

        assert!(result.is_ok(), "Failed to setup client: {:?}", result.err());
    }

    #[test]
    fn test_successful_csp_handling() {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'; script-src 'self' https://trusted.com"),
        );
        let tag_name = "script";
        let request_origin = url::Origin::Tuple(
            "https".to_string(),
            url::Host::Domain("www.example.com".to_string()),
            443,
        );

        let csp_test = handle_csp(&headers, tag_name, &request_origin);

        assert!(
            csp_test.is_ok(),
            "CSP handling failed: {:?}",
            csp_test.err()
        );
    }

    #[test]
    fn test_failed_csp_handling() {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'none'; script-src 'none'"),
        );
        let tag_name = "script";
        let request_origin = url::Origin::Tuple(
            "https".to_string(),
            url::Host::Domain("www.example.com".to_string()),
            443,
        );

        let csp_test = handle_csp(&headers, tag_name, &request_origin);

        assert!(
            csp_test.is_err(),
            "CSP handling should have failed but passed: {:?}",
            csp_test
        );
    }

    #[test]
    fn test_cors() {
        let mut response_headers = HeaderMap::new();
        response_headers.insert(
            ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static("https://www.example.com"),
        );
        response_headers.insert(
            ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET, POST, OPTIONS"),
        );
        response_headers.insert(
            ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("Content-Type, Authorization"),
        );

        let request_origin = url::Origin::Tuple(
            "https".to_string(),
            url::Host::Domain("www.example.com".to_string()),
            443,
        );

        let request_method = Method::POST;
        let request_headers = HeaderMap::new();

        let result = validate_cors_preflight(
            &response_headers,
            &request_origin,
            &request_method,
            &request_headers,
        );

        assert!(
            result.is_ok(),
            "CORS preflight validation failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_cors_violation() {
        let mut response_headers = HeaderMap::new();
        response_headers.insert(
            ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static("https://www.example.com"),
        );
        response_headers.insert(
            ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET, POST, OPTIONS"),
        );
        // Missing Access-Control-Allow-Headers

        let request_origin = url::Origin::Tuple(
            "https".to_string(),
            url::Host::Domain("www.example.com".to_string()),
            443,
        );

        let request_method = Method::POST;
        let request_headers = HeaderMap::new();

        let result = validate_cors_preflight(
            &response_headers,
            &request_origin,
            &request_method,
            &request_headers,
        );

        assert!(
            result.is_err(),
            "CORS preflight validation should have failed but passed: {:?}",
            result
        );
    }
}
