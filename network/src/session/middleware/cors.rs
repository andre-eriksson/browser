use http::{
    HeaderMap, Method,
    header::{
        ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
        ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
    },
};
use url::{Origin, Url};

use crate::{
    http::{request::Credentials, response::Response},
    session::middleware::simple::{is_simple_header, is_simple_headers, is_simple_method},
};

/// Determines if a CORS request is allowed based on the preflight response.
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
/// * `bool` - True if the CORS request is allowed, false otherwise.
pub fn is_cors_allowed(
    request_origin: &Origin,
    request_credentials: &Credentials,
    request_url: &Url,
    request_method: &Method,
    request_headers: &HeaderMap,
    preflight_response: Response,
) -> bool {
    // TODO: Return an Result<> with error details instead of just bool

    if request_origin.ascii_serialization() == request_url.origin().ascii_serialization() {
        return true;
    }

    let request_origin = request_origin.ascii_serialization();

    let allowed_credentials = preflight_response
        .headers
        .get(ACCESS_CONTROL_ALLOW_CREDENTIALS);

    if let Some(cred) = &allowed_credentials
        && request_credentials == &Credentials::Include
        && cred.to_str().unwrap_or("") != "true"
    {
        return false;
    }

    let allowed_origin = preflight_response.headers.get(ACCESS_CONTROL_ALLOW_ORIGIN);

    if let Some(origin) = allowed_origin {
        let origin_str = origin.to_str().unwrap_or("");

        if let Some(cred) = allowed_credentials {
            match request_credentials {
                Credentials::Include => {
                    if origin_str == "*" {
                        return false;
                    }
                    if cred.to_str().unwrap_or("") != "true" {
                        return false;
                    }
                    if origin_str != request_origin {
                        return false;
                    }
                }
                Credentials::SameOrigin => {
                    // Same-origin requests are already allowed
                    // So here we are in a cross-origin request => reject
                    return false;
                }
                Credentials::Omit => {
                    // Allow any origin including "*"
                }
            }
        }

        if request_origin == "null" && origin_str != "null" && origin_str != "*" {
            return false;
        }

        if origin != "*" && origin_str != request_origin {
            return false;
        }
    } else {
        return false;
    }

    let allowed_methods = preflight_response.headers.get(ACCESS_CONTROL_ALLOW_METHODS);

    if let Some(methods) = allowed_methods {
        let methods_str = methods.to_str().unwrap_or("");
        let method_allowed = methods_str
            .split(',')
            .any(|m| m.trim().eq_ignore_ascii_case(request_method.as_str()));

        if !method_allowed {
            return false;
        }
    } else if !is_simple_method(request_method) {
        return false;
    }

    let allowed_headers = preflight_response.headers.get(ACCESS_CONTROL_ALLOW_HEADERS);

    if let Some(headers) = allowed_headers {
        // TODO: Handle invalid headers instead of unwrap_or
        let headers_str = headers.to_str().unwrap_or("").to_lowercase();

        for (name, value) in request_headers.iter() {
            let name_str = name.as_str().trim().to_lowercase();

            if is_simple_header(&name_str, value) {
                continue;
            }

            if !headers_str
                .split(',')
                .map(|h| h.trim().to_ascii_lowercase())
                .any(|h| h == name_str)
            {
                return false;
            }
        }
    } else if !is_simple_headers(request_headers) {
        return false;
    }

    true
}
