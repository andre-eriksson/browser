use http::{
    HeaderMap, HeaderValue, Method,
    header::{
        ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
    },
};
use url::Origin;

/// Handles CORS preflight requests by checking the Access-Control-Allow headers
///
/// # Arguments
/// * `response` - The response from the preflight request
/// * `request_origin` - The origin of the request
///
/// # Returns
/// * `Ok(())` if the preflight request is valid
/// * `Err(String)` if there is a CORS violation
fn handle_origin(
    request_origin: &Origin,
    access_control_allow_origin: &HeaderValue,
) -> Result<(), String> {
    if access_control_allow_origin.to_str().unwrap_or("unknown")
        != request_origin.unicode_serialization()
    {
        return Err(format!(
            "CORS violation: Origin {} not allowed",
            access_control_allow_origin.to_str().unwrap_or("unknown")
        ));
    }

    Ok(())
}

/// Handles the Access-Control-Allow-Methods header
///
/// # Arguments
/// * `request_method` - The HTTP method of the request
/// * `access_control_allow_methods` - The value of the Access-Control-Allow-Methods header
///
/// # Returns
/// * `Ok(())` if the method is allowed
/// * `Err(String)` if the method is not allowed
fn handle_method(
    request_method: &Method,
    access_control_allow_methods: &HeaderValue,
) -> Result<(), String> {
    let allowed_methods = access_control_allow_methods
        .to_str()
        .unwrap_or("")
        .split(", ")
        .map(|s| s.trim())
        .collect::<Vec<&str>>();

    if !allowed_methods.contains(&request_method.as_str()) {
        return Err(format!(
            "CORS violation: Method {} not allowed",
            request_method
        ));
    }

    Ok(())
}

/// Handles the Access-Control-Allow-Headers header
///
/// # Arguments
/// * `request_headers` - The headers from the request
/// * `access_control_allow_headers` - The value of the Access-Control-Allow-Headers header
///
/// # Returns
/// * `Ok(())` if all headers are allowed
/// * `Err(String)` if any header is not allowed
fn handle_headers(
    request_headers: &HeaderMap<HeaderValue>,
    access_control_allow_headers: &HeaderValue,
) -> Result<(), String> {
    let allowed_headers = access_control_allow_headers
        .to_str()
        .unwrap_or("")
        .split(", ")
        .map(|s| s.trim())
        .collect::<Vec<&str>>();

    for (header_name, _) in request_headers.iter() {
        if !allowed_headers.contains(&header_name.as_str()) {
            return Err(format!(
                "CORS violation: Header {} not allowed",
                header_name
            ));
        }
    }

    Ok(())
}

/// Handles CORS preflight requests by checking the Access-Control-Allow headers
///
/// # Arguments
/// * `response` - The response from the preflight request
/// * `request_origin` - The origin of the request
/// * `request_method` - The HTTP method of the request
/// * `request_headers` - The headers from the request
///
/// # Returns
/// * `Ok(())` if the preflight request is valid
/// * `Err(String)` if there is a CORS violation
pub fn validate_cors_preflight(
    response_headers: &HeaderMap<HeaderValue>,
    request_origin: &Origin,
    request_method: &Method,
    request_headers: &HeaderMap<HeaderValue>,
) -> Result<(), String> {
    let access_control_allow_origin = response_headers.get(ACCESS_CONTROL_ALLOW_ORIGIN);
    let access_control_allow_method = response_headers.get(ACCESS_CONTROL_ALLOW_METHODS);
    let access_control_allow_headers = response_headers.get(ACCESS_CONTROL_ALLOW_HEADERS);

    if let Some(allow_origin) = access_control_allow_origin {
        handle_origin(request_origin, allow_origin)?;
    } else {
        return Err("CORS violation: No Access-Control-Allow-Origin header found".to_string());
    }

    if let Some(allow_methods) = access_control_allow_method {
        handle_method(request_method, allow_methods)?;
    } else {
        return Err("CORS violation: No Access-Control-Allow-Methods header found".to_string());
    }

    if let Some(allow_headers) = access_control_allow_headers {
        handle_headers(request_headers, allow_headers)?;
    } else {
        return Err("CORS violation: No Access-Control-Allow-Headers header found".to_string());
    }

    Ok(())
}
