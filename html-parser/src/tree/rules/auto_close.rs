/// A utility function to determine if a tag should automatically close based on the current tag and the new tag being encountered.
///
/// # Arguments
/// * `current_tag` - The name of the current tag that is open.
/// * `new_tag` - The name of the new tag that is being encountered.
///
/// # Returns
/// A boolean indicating whether the current tag should be automatically closed when the new tag is encountered.
pub fn should_auto_close(current_tag: &str, new_tag: &str) -> bool {
    let current_lower = current_tag;
    let new_lower = new_tag;

    match current_lower {
        "p" => {
            matches!(
                new_lower,
                "div"
                    | "p"
                    | "h1"
                    | "h2"
                    | "h3"
                    | "h4"
                    | "h5"
                    | "h6"
                    | "ul"
                    | "ol"
                    | "li"
                    | "dl"
                    | "dt"
                    | "dd"
                    | "blockquote"
                    | "pre"
                    | "form"
                    | "table"
                    | "section"
                    | "article"
                    | "aside"
                    | "header"
                    | "footer"
                    | "nav"
                    | "main"
                    | "figure"
                    | "hr"
            )
        }
        "li" => new_lower == "li",
        "dd" | "dt" => {
            matches!(new_lower, "dd" | "dt")
        }
        "option" => {
            matches!(new_lower, "option" | "optgroup")
        }
        "tr" => new_lower == "tr",
        "td" | "th" => {
            matches!(new_lower, "td" | "th" | "tr")
        }
        _ => false,
    }
}
