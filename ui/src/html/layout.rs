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
        | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "hr" => ElementType::Block,

        "span" | "a" | "strong" | "em" | "i" | "b" | "u" | "code" | "small" | "sub" | "sup"
        | "img" => ElementType::Inline,

        "script" | "style" => ElementType::Skip, // TODO: Handle script/style elements

        _ => {
            debug!("Unknown tag name: {}", tag_name);
            ElementType::Unknown
        }
    }
}

/// Returns the margin for a given HTML element based on its tag name.
///
/// # Arguments
/// * `tag_name` - The name of the HTML tag (e.g., "body", "h1", "div").
///
/// # Returns
/// * `egui::Margin` - The margin to be applied to the element, which can be used for layout purposes.
pub fn get_margin_for_element(tag_name: &str) -> egui::Margin {
    match tag_name {
        "body" => egui::Margin::same(8),
        "h1" => egui::Margin::symmetric(0, 8),
        "h2" => egui::Margin::symmetric(0, 7),
        "h3" => egui::Margin::symmetric(0, 6),
        "h4" => egui::Margin::symmetric(0, 7),
        "h5" => egui::Margin::symmetric(0, 8),
        "h6" => egui::Margin::symmetric(0, 9),
        "div" => egui::Margin::same(0),
        "pre" => egui::Margin::symmetric(0, 13),
        _ => egui::Margin::symmetric(0, 4),
    }
}
