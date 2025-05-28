use logos::Logos;

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
