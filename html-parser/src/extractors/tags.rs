/// Extracts the tag name from a given HTML tag slice.
///
/// # Arguments
/// * `tag_slice` - A string slice representing the HTML tag, e.g., `<div class="example">`.
///
/// # Returns
/// A string slice containing the tag name, e.g., `div` for `<div>`, or `span` for `<span class="example">`.
pub fn extract_tag_name(tag_slice: &str) -> &str {
    let tag_name_end = tag_slice
        .find(|c: char| c.is_whitespace() || c == '>')
        .unwrap_or(tag_slice.len());
    let tag_name_start = if tag_slice.starts_with("</") {
        2 // Skip "</"
    } else {
        1 // Skip "<"
    };

    &tag_slice[tag_name_start..tag_name_end]
}
