use once_cell::sync::OnceCell;
use regex::Regex;

/// This module provides lazy static regex patterns for matching HTML doctype declarations and XML declarations.
/// It uses `OnceCell` to initialize the regex patterns only once, ensuring efficient memory usage and performance.
pub mod patterns {
    use super::*;

    /// A lazy static regex for matching HTML doctype declarations.
    pub static DOCTYPE_REGEX: OnceCell<Regex> = OnceCell::new();

    /// A lazy static regex for matching XML declarations.
    pub static XML_DECLARATION_REGEX: OnceCell<Regex> = OnceCell::new();
}

/// This struct provides methods to access regex patterns for HTML doctype and XML declarations.
pub struct Patterns;

impl Patterns {
    /// The regex for matching HTML doctype declarations.
    ///
    /// # Returns
    /// A reference to a `Regex` instance that matches HTML doctype declarations.
    pub fn doctype_regex() -> &'static Regex {
        patterns::DOCTYPE_REGEX.get_or_init(|| {
            Regex::new(r#"<!DOCTYPE\s+([a-zA-Z0-9]+)(?:\s+PUBLIC\s+\"([^\"]*)\"\s+\"([^\"]*)\")?>"#)
                .unwrap()
        })
    }

    /// The regex for matching XML declarations.
    ///
    /// # Returns
    /// A reference to a `Regex` instance that matches XML declarations.
    pub fn xml_declaration_regex() -> &'static Regex {
        patterns::XML_DECLARATION_REGEX.get_or_init(|| {
            Regex::new(
                r#"^<\?xml\s+version=\"([^\"]+)\"(?:\s+encoding=\"([^\"]+)\")?(?:\s+standalone=\"(yes|no)\")?\s*\?>"#,
            )
            .unwrap()
        })
    }
}
