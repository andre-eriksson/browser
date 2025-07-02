pub fn resolve_image_path(url: &str, src_value: &String) -> String {
    let image_url = if src_value.starts_with("http") {
        src_value.to_string()
    } else if src_value.starts_with('/') {
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

pub fn is_inline_element(tag_name: &str) -> bool {
    matches!(
        tag_name.to_lowercase().as_str(),
        "span" | "a" | "strong" | "em" | "i" | "b" | "u" | "small" | "sub" | "sup" | "code" | "img"
    )
}

pub fn get_depth_color(depth: usize) -> egui::Color32 {
    // Generate a color based on the depth, cycling through a palette
    let colors = [
        egui::Color32::from_rgb(255, 100, 100), // Bright red
        egui::Color32::from_rgb(100, 255, 100), // Bright green
        egui::Color32::from_rgb(100, 150, 255), // Bright blue
        egui::Color32::from_rgb(255, 200, 100), // Orange
        egui::Color32::from_rgb(100, 255, 255), // Cyan
        egui::Color32::from_rgb(200, 100, 255), // Purple
    ];
    colors[depth % colors.len()]
}
