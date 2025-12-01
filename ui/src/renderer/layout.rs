use html_syntax::{HtmlTag, KnownTag};

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
pub fn get_element_type(tag: &HtmlTag) -> ElementType {
    if let HtmlTag::Known(tag_name) = tag {
        return get_element_type_known(tag_name);
    }

    ElementType::Unknown
}

fn get_element_type_known(tag: &KnownTag) -> ElementType {
    match tag {
        KnownTag::Div
        | KnownTag::P
        | KnownTag::H1
        | KnownTag::H2
        | KnownTag::H3
        | KnownTag::H4
        | KnownTag::H5
        | KnownTag::H6
        | KnownTag::Hr
        | KnownTag::Address
        | KnownTag::Fieldset
        | KnownTag::Form
        | KnownTag::Legend
        | KnownTag::Nav
        | KnownTag::Ul
        | KnownTag::Ol
        | KnownTag::Details
        | KnownTag::Table
        | KnownTag::Thead
        | KnownTag::Tbody
        | KnownTag::Tr
        | KnownTag::Figcaption
        | KnownTag::Dl
        | KnownTag::Dt
        | KnownTag::Dd => ElementType::Block,

        KnownTag::Li | KnownTag::Summary => ElementType::ListItem,

        KnownTag::Span
        | KnownTag::A
        | KnownTag::Strong
        | KnownTag::Em
        | KnownTag::I
        | KnownTag::B
        | KnownTag::U
        | KnownTag::Code
        | KnownTag::Small
        | KnownTag::Sub
        | KnownTag::Sup
        | KnownTag::Img
        | KnownTag::Time
        | KnownTag::Label
        | KnownTag::Abbr
        | KnownTag::Input
        | KnownTag::Textarea
        | KnownTag::Th
        | KnownTag::Td => ElementType::Inline,

        KnownTag::Script | KnownTag::Style => ElementType::Skip, // TODO: Handle script/style elements

        _ => ElementType::Unknown,
    }
}
