use logos::Logos;

/// Represents the tokens used in an HTML document parser.
/// This enum defines the various types of tokens that can be recognized in an HTML document,
/// including XML declarations, doctype declarations, comments, tags, and fallback tokens.
///
/// # Token Types
/// * `XmlDeclaration` - Matches XML declaration tags.
/// * `Doctype` - Matches doctype declarations.
/// * `Comment` - Matches HTML comments.
/// * `StartTag` - Matches opening tags without attributes.
/// * `EndTag` - Matches closing tags.
/// * `StartTagWithAttributes` - Matches opening tags with attributes.
/// * `Text` - Matches text content outside of tags.
/// * `Unknown` - Matches any character that does not fit into the other categories.
#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(skip r"[ \t\n\r]+")]
pub enum Token {
    #[regex(r"<\\?xml[^?]*\\?>")]
    XmlDeclaration,

    #[regex(r"<!DOCTYPE[^>]*>")]
    Doctype,

    #[regex(r"<!--[^-]*(?:-[^-]+)*-->")]
    Comment,

    // === Tags ===
    #[regex(r"<[a-zA-Z][a-zA-Z0-9-]*>", priority = 2)]
    StartTag,

    #[regex(r"</[a-zA-Z][a-zA-Z0-9-]*>", priority = 2)]
    EndTag,

    #[regex(r"<[a-zA-Z][a-zA-Z0-9-]*\s+[^>]*>", priority = 2)]
    StartTagWithAttributes,

    // === Fallback Tokens ===
    #[regex(r"[^<]+", priority = 1)]
    Text,

    #[regex(r".", priority = 0)]
    Unknown,
}
