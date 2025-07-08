use api::dom::ConcurrentElement;

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
/// use crate::html::util::resolve_path;
///
/// let current_url = "http://example.com/page";
/// let relative_path = "images/photo.jpg";
/// let absolute_path = "/images/photo.jpg";
/// let full_url = "http://example.com/images/photo.jpg";
///
/// assert_eq!(resolve_path(current_url, &relative_path.to_string()), "http://example.com/images/photo.jpg");
/// assert_eq!(resolve_path(current_url, &absolute_path.to_string()), "http://example.com/page/images/photo.jpg");
/// assert_eq!(resolve_path(current_url, &full_url.to_string()), "http://example.com/images/photo.jpg");
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

        // Absolute path relative to domain
        let base_url = if let Some(pos) = url.find("://") {
            if let Some(domain_end) = url[pos + 3..].find('/') {
                &url[..pos + 3 + domain_end]
            } else {
                url
            }
        } else {
            url
        };
        format!("{}{}", base_url, src_value)
    } else {
        // Relative path
        let base_url = if url.ends_with('/') {
            url.to_string()
        } else {
            // Remove filename from URL to get directory
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

/// Returns a color for a given `ConcurrentElement` based on its ID.
///
/// # Arguments
/// * `element` - A reference to the `ConcurrentElement` whose color is to be determined.
///
/// # Returns
/// * `egui::Color32` - A color corresponding to the element's ID, cycling through a predefined set of colors.
pub fn get_color(element: &ConcurrentElement) -> egui::Color32 {
    let colors = [
        egui::Color32::from_rgb(255, 100, 100), // Bright red
        egui::Color32::from_rgb(100, 255, 100), // Bright green
        egui::Color32::from_rgb(100, 150, 255), // Bright blue
        egui::Color32::from_rgb(255, 200, 100), // Orange
        egui::Color32::from_rgb(100, 255, 255), // Cyan
        egui::Color32::from_rgb(200, 100, 255), // Purple
        egui::Color32::from_rgb(255, 150, 150), // Light red
        egui::Color32::from_rgb(150, 255, 150), // Light green
        egui::Color32::from_rgb(150, 200, 255), // Light blue
        egui::Color32::from_rgb(255, 255, 150), // Light yellow
        egui::Color32::from_rgb(255, 150, 255), // Light magenta
        egui::Color32::from_rgb(150, 255, 200), // Light cyan
        egui::Color32::from_rgb(200, 150, 255), // Light purple
        egui::Color32::from_rgb(255, 200, 150), // Light orange
    ];
    colors[element.id as usize % colors.len()]
}
