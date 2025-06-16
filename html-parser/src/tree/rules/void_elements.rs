/// A utility function to check if a given HTML tag name is a void element.
///
/// # Arguments
/// * `tag_name` - The name of the HTML tag to check.
///
/// [Void elements](https://html.spec.whatwg.org/#void-elements)
pub fn is_void_element(tag_name: &str) -> bool {
    matches!(
        tag_name.to_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "source"
            | "track"
            | "wbr"
    )
}
