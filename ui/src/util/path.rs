/// Resolves a relative or absolute image path based on the current URL and the source value.
///
/// # Arguments
/// * `base_url` - The base URL of the current page.
/// * `path` - A path that can be a relative path, absolute path, or full URL.
///
/// # Returns
/// * `String` - The resolved URL, which can be a full URL or a path relative to the current page.
///
/// # Example
/// ```rust
/// use ui::util::path::resolve_path;
///
/// let current_url: &str = "https://example.com/page";
/// let relative_path: &str = "images/photo.jpg";
/// let absolute_path: &str = "/images/photo.jpg";
/// let full_url: &str = "https://website.com/images/photo.jpg";
///
/// assert_eq!(resolve_path(current_url, relative_path), "https://example.com/images/photo.jpg");
/// assert_eq!(resolve_path(current_url, absolute_path), "https://example.com/images/photo.jpg");
/// assert_eq!(resolve_path(current_url, full_url), "https://website.com/images/photo.jpg");
/// ```
pub fn resolve_path(base_url: &str, path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }

    if path.starts_with("//") {
        let protocol = if base_url.starts_with("https") {
            "https:"
        } else {
            "http:"
        };
        return format!("{}{}", protocol, path);
    }

    // Parse the base URL to extract scheme, host, and path
    let (scheme_and_host, base_path) = if let Some(scheme_end) = base_url.find("://") {
        let after_scheme = &base_url[scheme_end + 3..];
        if let Some(path_start) = after_scheme.find('/') {
            let scheme_and_host = &base_url[..scheme_end + 3 + path_start];
            let base_path = &base_url[scheme_end + 3 + path_start..];
            (scheme_and_host, base_path)
        } else {
            (base_url, "")
        }
    } else {
        (base_url, "")
    };

    if path.starts_with('/') {
        return format!("{}{}", scheme_and_host, path);
    }

    let mut result_path = if base_path.is_empty() || !base_path.contains('/') {
        String::new()
    } else if let Some(last_slash) = base_path.rfind('/') {
        base_path[..last_slash + 1].to_string()
    } else {
        String::new()
    };

    if !result_path.ends_with('/') && !path.is_empty() {
        result_path.push('/');
    }
    result_path.push_str(path);

    format!("{}{}", scheme_and_host, result_path)
}
