use once_cell::sync::OnceCell;
use regex::Regex;

pub mod patterns {
    use super::*;

    /// A lazy static regex for matching HTML doctype declarations.
    pub static DOCTYPE_REGEX: OnceCell<Regex> = OnceCell::new();

    /// A lazy static regex for matching XML declarations.
    pub static XML_DECLARATION_REGEX: OnceCell<Regex> = OnceCell::new();
}

pub struct Patterns;

impl Patterns {
    pub fn doctype_regex() -> &'static Regex {
        patterns::DOCTYPE_REGEX.get_or_init(|| {
            Regex::new(r#"<!DOCTYPE\s+([a-zA-Z0-9]+)(?:\s+PUBLIC\s+\"([^\"]*)\"\s+\"([^\"]*)\")?>"#)
                .unwrap()
        })
    }

    pub fn xml_declaration_regex() -> &'static Regex {
        patterns::XML_DECLARATION_REGEX.get_or_init(|| {
            Regex::new(
                r#"^<\?xml\s+version=\"([^\"]+)\"(?:\s+encoding=\"([^\"]+)\")?(?:\s+standalone=\"(yes|no)\")?\s*\?>"#,
            )
            .unwrap()
        })
    }
}
