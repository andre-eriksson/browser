use http::{
    HeaderMap, HeaderValue, Method,
    header::{ACCEPT, ACCEPT_LANGUAGE, CONTENT_LANGUAGE, CONTENT_TYPE, RANGE},
};

/// Checks if the Content-Type is a simple type.
///
/// # Arguments
/// * `content_type` - The Content-Type header value to check.
fn is_simple_content_type(content_type: &str) -> bool {
    let content_type_lower = content_type.to_lowercase();
    let media_type = content_type_lower.split(';').next().unwrap_or("").trim();

    matches!(
        media_type,
        "application/x-www-form-urlencoded" | "multipart/form-data" | "text/plain"
    )
}

/// Checks if the request is a simple request.
///
/// # Arguments
/// * `headers` - The headers of the request.
/// * `http_method` - The HTTP method of the request.
///
/// # Returns
/// `true` if the request is a simple request, `false` otherwise.
pub fn is_simple_request(headers: HeaderMap<HeaderValue>, http_method: Method) -> bool {
    // Check allowed headers for simple requests
    let allowed_headers = [
        ACCEPT,
        ACCEPT_LANGUAGE,
        CONTENT_LANGUAGE,
        CONTENT_TYPE,
        RANGE,
    ];

    for (header_name, _) in headers.iter() {
        if !allowed_headers.contains(header_name) {
            return false;
        }
    }

    // Check range header for a single byte range
    if let Some(range) = headers.get(RANGE) {
        if let Ok(range_str) = range.to_str() {
            if !range_str.starts_with("bytes=")
                || range_str.split('=').nth(1).unwrap_or("").contains(',')
            {
                return false; // Range header is not a simple byte range
            }
        } else {
            return false; // Range header is not valid UTF-8
        }
    }

    match http_method {
        Method::GET | Method::HEAD => true,
        Method::POST => {
            let content_type = headers.get(CONTENT_TYPE);

            if let Some(content_type) = content_type {
                if let Ok(content_type) = content_type.to_str() {
                    return is_simple_content_type(content_type);
                }
            }
            // Default to text/plain if no Content-Type is specified
            true
        }
        _ => false, // Other methods are not simple
    }
}
