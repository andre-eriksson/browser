use tracing::debug;

/// Represents the type of an HTML element.
///
/// # Variants
/// * `Block` - Represents a block-level element (e.g., `<div>`, `<p>`, `<h1>`).
/// * `Inline` - Represents an inline element (e.g., `<span>`, `<a>`, `<strong>`).
/// * `Skip` - Represents an element that should be skipped during rendering (e.g., `<script>`, `<style>`).
/// * `Unknown` - Represents an unknown or unrecognized element type, i.e. not yet implemented.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementType {
    Block,
    Inline,
    ListItem,
    Skip,
    Unknown,
}

/// Determines the type of an HTML element based on its tag name.
///
/// # Arguments
/// * `tag_name` - The name of the HTML tag (e.g., "div", "span", "h1").
///
/// # Returns
/// * `ElementType` - The type of the element, which can be Block, Inline, Skip, or Unknown.
pub fn get_element_type(tag_name: &str) -> ElementType {
    match tag_name {
        "body" | "div" | "header" | "footer" | "main" | "section" | "article" | "aside" | "pre"
        | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "hr" | "address" | "fieldset"
        | "form" | "legend" | "nav" | "ul" | "ol" | "details" | "table" | "thead" | "tbody"
        | "tr" | "figcaption" | "dl" | "dt" | "dd" => ElementType::Block,
        "li" | "summary" => ElementType::ListItem,
        "span" | "a" | "strong" | "em" | "i" | "b" | "u" | "code" | "small" | "sub" | "sup"
        | "img" | "time" | "label" | "abbr" | "input" | "textarea" | "th" | "td" => {
            ElementType::Inline
        }

        "script" | "style" => ElementType::Skip, // TODO: Handle script/style elements

        _ => {
            debug!("Unknown tag name: {}", tag_name);
            ElementType::Unknown
        }
    }
}
