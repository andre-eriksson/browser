use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserState {
    /// The initial state, where the parser is reading regular text
    ///
    /// # Example
    /// ```html
    /// Hello, World!
    Data,

    /// Represents the start of a tag ('<')
    ///
    /// # Example
    /// ```html
    /// <tag>
    TagOpen,

    /// Represents the start of an end tag ('</')
    ///
    /// # Example
    /// ```html
    /// </tag>
    EndTagOpen,

    /// Represents the start of a self-closing tag ('<tag/>')
    ///
    /// # Example
    /// ```html
    /// <tagName />
    SelfClosingTagStart,

    /// Represents the tag name after an opening or closing tag
    ///
    /// # Example
    /// ```html
    /// <tagName>
    TagName,

    /// Represents the state after reading a tag name, before attributes
    ///
    /// # Example
    /// ```html
    /// <tagName >
    BeforeAttributeName,

    /// Represents the state of reading an attribute name
    ///
    /// # Example
    /// ```html
    /// <tagName attributeName>
    AttributeName,

    /// Represents the state after reading an attribute name, before the equal sign
    ///
    /// # Example
    /// ```html
    /// <tagName attributeName >
    AfterAttributeName,

    /// Represents the state of reading an equal sign before an attribute value
    ///
    /// # Example
    /// ```html
    /// <tagName attributeName= >
    BeforeAttributeValue,

    /// Represents the state of reading an attribute value
    ///
    /// # Example
    /// ```html
    /// <tagName attributeName="value">
    AttributeValueDoubleQuoted,

    /// Represents the state of reading a single-quoted attribute value
    ///
    /// # Example
    /// ```html
    /// <tagName attributeName='value'>
    AttributeValueSingleQuoted,

    /// Represents the state of reading an unquoted attribute value
    ///
    /// # Example
    /// ```html
    /// <tagName attributeName=value>
    AttributeValueUnquoted,

    /// Represents the state after reading a quoted attribute value
    ///
    /// # Example
    /// ```html
    /// <tagName attributeName="value" >
    AfterAttributeValueQuoted,

    /// Represents the start of a declaration (e.g., `<!` or `<?`)
    ///
    /// # Example
    /// ```html
    /// <!DOCTYPE html>
    /// <?xml version="1.0"?>
    StartDeclaration,

    /// Represents a bogus comment that does not follow the correct syntax
    ///
    /// # Example
    /// ```html
    /// <!—— This is a bogus comment ——>
    BogusComment,

    /// Represents the start of a comment ('<!--')
    ///
    /// # Example
    /// ```html
    /// <!--
    CommentStart,

    /// Represents the state of being inside a comment
    ///
    /// # Example
    /// ```html
    /// <!-- This is a comment -->
    Comment,

    /// Represents the end of a comment ('-->')
    ///
    /// # Example
    /// ```html
    /// <!-- This is a comment -->
    CommentEnd,

    /// Represents the state of being inside an XML declaration
    ///
    /// # Example
    /// ```html
    /// <?xml version="1.0"?>
    XmlDeclaration,

    /// Represents the state of being inside a doctype declaration
    ///
    /// # Example
    /// ```html
    /// <!DOCTYPE html>
    DoctypeDeclaration,

    /// Represents the state of being inside a script tag
    ///
    /// # Example
    /// ```html
    /// <script>
    ///   console.log("Hello, World!");
    /// </script>
    ScriptData,

    /// Represents the start of a script end tag ('</script>')
    ///
    /// # Example
    /// ```html
    /// <script>
    ///   console.log("Hello, World!");
    /// </script>
    ScriptDataEndTagOpen,
}

/// Represents the kind of token being parsed in the HTML document
///
/// # Fields
/// * `StartTag` - Represents the start of an HTML tag (e.g., `<div>`).
/// * `EndTag` - Represents the end of an HTML tag (e.g., `</div>`).
/// * `Comment` - Represents an HTML comment (e.g., `<!-- comment -->`).
/// * `Text` - Represents plain text content within the HTML document.
/// * `DoctypeDeclaration` - Represents a doctype declaration (e.g., `<!DOCTYPE html>`).
/// * `XmlDeclaration` - Represents an XML declaration (e.g., `<?xml version="1.0"?>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    StartTag,
    EndTag,
    Comment,
    Text,
    DoctypeDeclaration,
    XmlDeclaration,
}

/// Represents a token in the HTML document, including its kind, data, and attributes
///
/// # Fields
/// * `kind` - The type of token (e.g., start tag, end tag, comment).
/// * `data` - The content of the token, such as the tag name or text content.
/// * `attributes` - A map of attributes associated with the token, where the key is the attribute name and the value is the attribute value.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub data: String,
    pub attributes: HashMap<String, String>,
}
