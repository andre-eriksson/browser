use http::{HeaderMap, HeaderValue, Method};

pub struct SimpleMiddleware;

impl SimpleMiddleware {
    /// Determines if a request is a simple request according to CORS specifications.
    ///
    /// # Arguments
    /// * `method` - The HTTP method of the request.
    /// * `headers` - The headers of the request.
    ///
    /// # Returns
    /// * `bool` - True if the request is a simple request, false otherwise.
    pub fn is_simple_request(method: &Method, headers: &HeaderMap) -> bool {
        Self::is_simple_method(method) && Self::is_simple_headers(headers)
    }

    /// Determines if the HTTP method is a simple method according to CORS specifications.
    ///
    /// # Arguments
    /// * `method` - The HTTP method to check.
    ///
    /// # Returns
    /// * `bool` - True if the method is simple, false otherwise.
    pub fn is_simple_method(method: &Method) -> bool {
        let simple_methods = [Method::GET, Method::HEAD, Method::POST];
        simple_methods.contains(method)
    }

    /// Determines if the headers are all simple headers according to CORS specifications.
    ///
    /// # Arguments
    /// * `headers` - The headers to check.
    ///
    /// # Returns
    /// * `bool` - True if all headers are simple, false otherwise.
    pub fn is_simple_headers(headers: &http::HeaderMap) -> bool {
        for (name, value) in headers.iter() {
            let name_str = name.as_str().to_lowercase();
            if name_str != "accept"
                && name_str != "accept-language"
                && name_str != "content-language"
                && name_str != "content-type"
                && name_str != "range"
            {
                return false;
            }

            if !Self::is_simple_header(name.as_str(), value) {
                return false;
            }
        }

        true
    }

    /// Determines if a specific header is a simple header according to CORS specifications.
    ///
    /// # Arguments
    /// * `header_name` - The name of the header.
    /// * `header_value` - The value of the header.
    ///
    /// # Returns
    /// * `bool` - True if the header is simple, false otherwise.
    pub fn is_simple_header(header_name: &str, header_value: &HeaderValue) -> bool {
        let value_length = header_value.as_bytes().len();

        if value_length > 128 {
            return false;
        }

        match header_name {
            "accept" => {
                let val_bytes = header_value.as_bytes();

                // Can't contain 0x00-0x1F except 0x09 (HT), "():<>?@[\]{} and 0x7F (DEL)"
                if val_bytes
                    .iter()
                    .any(|&b| (b <= 0x1F && b != 0x09) || b == 0x7F || b"():<>?@[]{}".contains(&b))
                {
                    return false;
                }
            }
            "accept-language" | "content-language" => {
                // Check for 0-9 a-z A-Z *,-.;= and spaces
                let val_str = match header_value.to_str() {
                    Ok(v) => v,
                    Err(_) => return false,
                };

                if !val_str
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || " *,-.;=".contains(c))
                {
                    return false;
                }
            }
            "content-type" => {
                let val_bytes = header_value.as_bytes();

                // Can't contain 0x00-0x1F except 0x09 (HT), "():<>?@[\]{} and 0x7F (DEL)"
                if val_bytes.iter().any(|&b| {
                    (b <= 0x1F && b != 0x09) || b == 0x7F || b"():<>?@[\\]{}".contains(&b)
                }) {
                    return false;
                }

                if !Self::is_simple_content_type(header_value) {
                    return false;
                }
            }
            "range" => {
                if !Self::is_simple_range(header_value) {
                    return false;
                }
            }
            _ => {
                return false;
            }
        }

        true
    }

    /// Checks if the Content-Type header value is a simple content type.
    ///
    /// # Arguments
    /// * `value` - The HeaderValue of the Content-Type header.
    ///
    /// # Returns
    /// * `bool` - True if the content type is simple, false otherwise.
    fn is_simple_content_type(value: &HeaderValue) -> bool {
        let val_str = match value.to_str() {
            Ok(v) => v,
            Err(_) => return false,
        };

        let simple_types = [
            "application/x-www-form-urlencoded",
            "multipart/form-data",
            "text/plain",
        ];

        let mime_type = val_str.split(';').next().unwrap().trim();
        simple_types.contains(&mime_type)
    }

    /// Checks if the Range header value is a simple range.
    ///
    /// # Arguments
    /// * `value` - The HeaderValue of the Range header.
    ///
    /// # Returns
    /// * `bool` - True if the range is simple, false otherwise.
    fn is_simple_range(value: &HeaderValue) -> bool {
        let val_str = match value.to_str() {
            Ok(v) => v,
            Err(_) => return false,
        };

        if !val_str.starts_with("bytes=") || val_str.contains(',') {
            return false;
        }

        let spec = &val_str["bytes=".len()..];
        if !spec.contains('-') {
            return false;
        }

        if let Some(start) = spec.strip_prefix('-') {
            !start.is_empty() && start.chars().all(|c| c.is_ascii_digit())
        } else {
            let mut parts = spec.splitn(2, '-');
            let start = parts.next().unwrap_or("");
            let end = parts.next().unwrap_or("");
            !start.is_empty()
                && !end.is_empty()
                && start.chars().all(|c| c.is_ascii_digit())
                && end.chars().all(|c| c.is_ascii_digit())
        }
    }
}
