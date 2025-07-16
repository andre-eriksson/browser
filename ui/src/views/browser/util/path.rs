#[allow(dead_code)]
/// Resolves a relative or absolute image path based on the current URL and the source value.
///
/// # Arguments
/// * `url` - The current URL of the page.
/// * `src_value` - The source value of the image, which can be a relative path, absolute path, or full URL.
///
/// # Returns
/// * `String` - The resolved image URL, which can be a full URL or a path relative to the current page.
///
/// # Example
/// ```rust
/// use ui_iced::views::browser::util::path::resolve_path;
///
/// let current_url = "https://example.com/page";
/// let relative_path = "images/photo.jpg";
/// let absolute_path = "/images/photo.jpg";
/// let full_url = "https://website.com/images/photo.jpg";
///
/// assert_eq!(resolve_path(current_url, &relative_path.to_string()), "https://example.com/images/photo.jpg");
/// assert_eq!(resolve_path(current_url, &absolute_path.to_string()), "https://example.com/page/images/photo.jpg");
/// assert_eq!(resolve_path(current_url, &full_url.to_string()), "https://website.com/images/photo.jpg");
/// ```
pub fn resolve_path(url: &str, src_value: &String) -> String {
    let image_url = if src_value.starts_with("http") {
        src_value.to_string()
    } else if src_value.starts_with('/') {
        if src_value.starts_with("//") {
            if url.starts_with("https") {
                return format!("https:{}", src_value);
            } else {
                return format!("http:{}", src_value);
            }
        }

        // Absolute path
        format!("{}{}", url, src_value)
    } else {
        // Relative path
        let base_url = if url.ends_with('/') {
            url.to_string()
        } else {
            if let Some(last_slash) = url.rfind('/') {
                format!("{}/", &url[..last_slash])
            } else {
                format!("{}/", url)
            }
        };
        format!("{}{}", base_url, src_value)
    };
    image_url
}
