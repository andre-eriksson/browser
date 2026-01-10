use std::collections::HashMap;

/// Represents the kind of token being parsed in the HTML document
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// Represents the start of an HTML tag (e.g., `<div>`).
    StartTag,

    /// Represents the end of an HTML tag (e.g., `</div>`).
    EndTag,

    /// Represents an HTML comment (e.g., `<!-- comment -->`).
    Comment,

    /// Represents plain text content within the HTML document.
    Text,

    /// Represents a doctype declaration (e.g., `<!DOCTYPE html>`).
    DoctypeDeclaration,

    /// Represents an XML declaration (e.g., `<?xml version="1.0"?>`).
    XmlDeclaration,
}

/// Represents a token in the HTML document, including its kind, data, and attributes
#[derive(Debug, Clone)]
pub struct Token {
    /// The type of token (e.g., start tag, end tag, comment).
    pub kind: TokenKind,

    /// The content of the token, such as the tag name or text content.
    pub data: String,

    /// A map of attributes associated with the token, where the key is the attribute name and the value is the attribute value.
    pub attributes: HashMap<String, String>,
}
