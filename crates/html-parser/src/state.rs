use std::collections::HashMap;

use html_dom::{BuildResult, Token, TokenState};

use crate::errors::HtmlParsingError;

/// Represents the type of resource that the parser is waiting for.
#[derive(Debug, Clone)]
pub enum ResourceType {
    Style,
    Favicon,
}

/// Metadata about a resource that the parser is waiting for.
#[derive(Debug, Clone, Default)]
pub struct ResourceMetadata {
    pub content_type: Option<String>,
    pub sizes: Option<(u32, u32)>,
}

/// Describes why the parser needs to block, including all data needed to construct the reason.
pub(crate) enum BlockingCause {
    Script,
    Style,
    Svg,
    Stylesheet {
        href: String,
    },
    Favicon {
        href: String,
        content_type: Option<String>,
        sizes: Option<(u32, u32)>,
    },
}

impl BlockingCause {
    /// Classifies the cause of blocking based on the current token state and the last token processed.
    pub(crate) fn classify_cause(current_state: TokenState, last_token: Option<&Token>) -> Option<Self> {
        match current_state {
            TokenState::ScriptData => Some(BlockingCause::Script),
            TokenState::StyleData => Some(BlockingCause::Style),
            TokenState::SvgData => Some(BlockingCause::Svg),
            TokenState::Data => {
                let attr = last_token?.attributes.as_ref()?;
                let rel = attr.get("rel")?.trim();
                let href = attr.get("href").cloned().unwrap_or_default();

                if rel.eq_ignore_ascii_case("stylesheet") {
                    Some(BlockingCause::Stylesheet { href })
                } else if rel.eq_ignore_ascii_case("icon") || rel.eq_ignore_ascii_case("shortcut icon") {
                    let content_type = attr.get("type").cloned();
                    let sizes = attr.get("sizes").and_then(|s| {
                        let (l, r) = s.split_once('x')?;
                        Some((l.parse().ok()?, r.parse().ok()?))
                    });
                    Some(BlockingCause::Favicon {
                        href,
                        content_type,
                        sizes,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Represents the script that is causing the parser to block, including all relevant data for inline and external scripts.
#[derive(Debug, Clone)]
pub enum Script {
    Inline {
        data: Result<String, HtmlParsingError>,
        type_attr: String,
    },

    External {
        src: String,
        is_async: bool,
        is_deferred: bool,
    },
}

/// Represents the reason why the HTML parser is blocked.
#[derive(Debug, Clone)]
pub enum BlockedReason {
    /// The parser is waiting for a script to load or execute.
    /// The associated `HashMap` contains attributes of the script element.
    WaitingForScript { script: Script },

    /// The parser is waiting for a style resource to load.
    /// The associated `HashMap` contains attributes of the style element.
    WaitingForStyle {
        data: Result<String, HtmlParsingError>,
        attributes: Option<HashMap<String, String>>,
    },

    /// The parser is waiting for a generic resource to load, from <link> tags.
    WaitingForResource(ResourceType, String, ResourceMetadata),

    /// The parser is waiting for SVG parsing to complete.
    SVGContent {
        data: Result<String, HtmlParsingError>,
    },
}

/// Represents the current state of the HTML parser.
#[derive(Default, Debug)]
pub enum ParserState<C> {
    /// The parser is actively processing input.
    #[default]
    Running,

    /// The parser is blocked, waiting for a specific reason.
    Blocked(BlockedReason),

    /// The parser has completed processing.
    Completed(BuildResult<C>),
}
