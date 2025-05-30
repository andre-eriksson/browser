use once_cell::sync::OnceCell;
use regex::Regex;

pub mod patterns {
    use super::*;

    /// A lazy static regex for matching HTML attributes.
    pub static ATTR_REGEX: OnceCell<Regex> = OnceCell::new();

    /// A lazy static regex for matching boolean attributes in HTML.
    pub static BOOL_ATTR_REGEX: OnceCell<Regex> = OnceCell::new();

    /// A lazy static regex for matching HTML doctype declarations.
    pub static DOCTYPE_REGEX: OnceCell<Regex> = OnceCell::new();

    /// A lazy static regex for matching XML declarations.
    pub static XML_DECLARATION_REGEX: OnceCell<Regex> = OnceCell::new();
}

pub fn init_regexes() {
    patterns::ATTR_REGEX
        .set(
            Regex::new(
                r#"([a-zA-Z_:][a-zA-Z0-9_:.-]*)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+))"#,
            )
            .unwrap(),
        )
        .expect("Failed to set ATTR_REGEX");

    patterns::BOOL_ATTR_REGEX
        .set(Regex::new(r#"\s+([a-zA-Z][a-zA-Z0-9-]*)(\s|>)"#).unwrap())
        .expect("Failed to set BOOL_ATTR_REGEX");

    patterns::DOCTYPE_REGEX
        .set(
            Regex::new(
                r#"<!DOCTYPE\s+([a-zA-Z0-9]+)(?:\s+PUBLIC\s+\"([^\"]*)\"\s+\"([^\"]*)\")?>"#,
            )
            .unwrap(),
        )
        .expect("Failed to set DOCTYPE_REGEX");

    patterns::XML_DECLARATION_REGEX
        .set(Regex::new(r#"^<\?xml\s+version=\"([^\"]+)\"(?:\s+encoding=\"([^\"]+)\")?(?:\s+standalone=\"(yes|no)\")?\s*\?>"#).unwrap())
        .expect("Failed to set XML_DECLARATION_REGEX");
}
