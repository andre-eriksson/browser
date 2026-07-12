//! CORS (Cross-Origin Resource Sharing) implementation for HTTP requests.
//!
//! # Specification
//! <https://fetch.spec.whatwg.org/#http-cors-protocol>

use http::{
    HeaderMap, Method,
    header::{
        ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS,
        ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, ORIGIN,
    },
};
use http_types::{
    properties::Credentials,
    request::{Request, RequestBuilder},
    response::HeaderResponse,
};
use url::{Origin, Url};

use crate::{
    errors::CorsError,
    simple::{is_simple_header, is_simple_headers, is_simple_method, is_simple_request},
};

/// Determines if a request is cross-origin based on the current origin and the
/// request origin.
///
/// # Arguments
/// * `current_origin` - The origin of the current document.
/// * `request_origin` - The origin of the request.
///
/// # Returns
/// True if the request is cross-origin, false otherwise.
pub fn is_cross_origin(current_origin: &Origin, request_origin: &Origin) -> bool {
    current_origin != request_origin
}

pub fn needs_preflight(
    current_url: Option<&Url>,
    request_url: &Url,
    request_headers: &HeaderMap,
    request_method: &Method,
) -> bool {
    let Some(url) = current_url else {
        return false;
    };

    if is_simple_request(request_method, request_headers) {
        return false;
    }

    is_cross_origin(&url.origin(), &request_url.origin())
}

/// Determines if a cross-origin request is allowed based on the preflight response
/// and the request details.
///
/// # Arguments
/// * `request_origin` - The origin of the request.
/// * `request_credentials` - The credentials mode of the request.
/// * `request_url` - The URL of the request.
/// * `request_method` - The HTTP method of the request.
/// * `request_headers` - The headers of the request.
/// * `preflight_response` - The response from the preflight CORS request.
///
/// # Returns
/// True if the request is allowed, false otherwise.
pub fn is_cross_origin_request_allowed(
    request_origin: &Origin,
    request_credentials: &Credentials,
    request_url: &Url,
    request_method: &Method,
    request_headers: &HeaderMap,
    preflight_response: &HeaderResponse,
) -> Result<(), CorsError> {
    if !preflight_response.status_code.is_success() {
        return Err(CorsError::InvalidPreflightResponse(format!(
            "returned a non-OK status code: '{}'",
            preflight_response.status_code
        )));
    }

    if request_origin.ascii_serialization() == request_url.origin().ascii_serialization() {
        return Ok(());
    }

    let request_origin = request_origin.ascii_serialization();

    let allowed_credentials = preflight_response
        .headers
        .get(ACCESS_CONTROL_ALLOW_CREDENTIALS);

    if let Some(cred) = allowed_credentials
        && request_credentials == &Credentials::Include
        && cred.to_str().unwrap_or_default() != "true"
    {
        return Err(CorsError::CredentialsNotAllowed);
    }

    let allowed_origin = preflight_response.headers.get(ACCESS_CONTROL_ALLOW_ORIGIN);

    if let Some(origin) = allowed_origin {
        let origin_str = origin.to_str().unwrap_or("");

        if allowed_credentials.is_some() {
            match request_credentials {
                Credentials::Include => {
                    if origin_str == "*" {
                        return Err(CorsError::CredentialNotAllowed(
                            "Include".to_string(),
                            format!("with wildcard origin '{request_origin}'"),
                        ));
                    }

                    if origin_str != request_origin {
                        return Err(CorsError::CredentialNotAllowed(
                            "Include".to_string(),
                            format!("for origin '{request_origin}'"),
                        ));
                    }
                }
                Credentials::SameOrigin => {
                    // Same-origin requests are already allowed
                    // So here we are in a cross-origin request => reject
                    return Err(CorsError::CredentialNotAllowed(
                        "Same-Origin".to_string(),
                        "for cross-origin requests".to_string(),
                    ));
                }
                Credentials::Omit => {
                    // Allow any origin including "*"
                }
            }
        }

        if request_origin == "null" && origin_str != "null" && origin_str != "*" {
            return Err(CorsError::InvalidOrigin(request_origin, origin_str.to_string()));
        }

        if origin != "*" && origin_str != request_origin {
            return Err(CorsError::InvalidOrigin(request_origin, origin_str.to_string()));
        }
    } else {
        return Err(CorsError::NoAccessControlAllowOrigin);
    }

    let allowed_methods = preflight_response.headers.get(ACCESS_CONTROL_ALLOW_METHODS);

    if let Some(methods) = allowed_methods {
        let methods_str = methods.to_str().unwrap_or("");
        let method_allowed = methods_str
            .split(',')
            .any(|m| m.trim().eq_ignore_ascii_case(request_method.as_str()));

        if !method_allowed {
            return Err(CorsError::InvalidMethod(request_method.as_str().to_string()));
        }
    } else if !is_simple_method(request_method) {
        return Err(CorsError::InvalidMethod(request_method.as_str().to_string()));
    }

    let allowed_headers = preflight_response.headers.get(ACCESS_CONTROL_ALLOW_HEADERS);

    if let Some(headers) = allowed_headers {
        // TODO: Handle invalid headers instead of unwrap_or
        let headers_str = headers.to_str().unwrap_or_default().to_lowercase();

        for (name, value) in request_headers {
            let name_str = name.as_str().trim().to_lowercase();

            if is_simple_header(name, value) {
                continue;
            }

            if !headers_str
                .split(',')
                .map(|h| h.trim().to_ascii_lowercase())
                .any(|h| h == name_str)
            {
                return Err(CorsError::InvalidHeader(name_str));
            }
        }
    } else if !is_simple_headers(request_headers) {
        return Err(CorsError::NonSimpleHeaders);
    }

    Ok(())
}

/// Creates a preflight CORS request based on the current URL, request headers,
/// request URL, and request method.
///
/// # Arguments
/// * `current_url` - The URL of the current document.
/// * `request_headers` - The headers of the request.
/// * `request_url` - The URL of the request.
/// * `request_method` - The HTTP method of the request.
///
/// # Returns
/// The constructed preflight CORS request.
pub fn make_preflight_request(
    current_url: &Url,
    request_headers: &HeaderMap,
    request_url: &Url,
    request_method: &Method,
) -> Request {
    let origin = current_url.origin();

    let mut request_headers = request_headers
        .iter()
        .map(|(k, _)| k.as_str().to_lowercase())
        .collect::<Vec<String>>();
    request_headers.sort();
    request_headers.dedup();
    let request_headers = request_headers.join(", ");

    let mut preflight_request = RequestBuilder::from(request_url.clone())
        .method(Method::OPTIONS)
        .header(ACCESS_CONTROL_REQUEST_HEADERS, &request_headers);

    if !matches!(origin, Origin::Opaque(_)) {
        preflight_request = preflight_request.header(ORIGIN, &origin.ascii_serialization());
    }

    if is_simple_method(request_method) {
        preflight_request.build()
    } else {
        preflight_request
            .header(ACCESS_CONTROL_REQUEST_METHOD, request_method.as_str())
            .build()
    }
}
