/// Checks if a given CSS property is an inherited property.
///
/// # Arguments
/// * `property` - A string slice representing the CSS property name.
///
/// # Returns
/// * `bool` - Returns true if the property is inherited, false otherwise.
pub fn is_inherited_property(property: &str) -> bool {
    // TODO: Add more inherited properties as needed
    matches!(
        property,
        "color" | "font-family" | "font-size" | "line-height" | "text-align"
    )
}
