/// Represents the various states the parser can be in.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserState {
    /// The initial state, where the parser is reading regular text
    ///
    /// # Example
    /// ```html
    /// Hello, World!
    #[default]
    Data,

    /// Represents the start of a tag (`<`)
    ///
    /// # Example
    /// ```html
    /// <tag>
    TagOpen,

    /// Represents the start of an end tag (`</`)
    ///
    /// # Example
    /// ```html
    /// </tag>
    EndTagOpen,

    /// Represents the start of a self-closing tag (`<tag/>`)
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

    /// Represents the start of a comment (`<!--`)
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

    /// Represents the end of a comment `-->`)
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
    /// ```js
    /// console.log("Hello, World!");
    ScriptData,

    /// Represents the state of being inside a style tag
    ///
    /// # Example
    /// ```css
    /// body {
    ///     background-color: #f0f0f0;
    /// }
    StyleData,
}
