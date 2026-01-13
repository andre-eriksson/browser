use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ResourceType {
    Style,
}

/// Represents the reason why the HTML parser is blocked.
#[derive(Debug, Clone)]
pub enum BlockedReason {
    /// The parser is waiting for a script to load or execute.
    /// The associated `HashMap` contains attributes of the script element.
    WaitingForScript(HashMap<String, String>),

    /// The parser is waiting for a style resource to load.
    /// The associated `HashMap` contains attributes of the style element.
    WaitingForStyle(HashMap<String, String>),

    /// The parser is waiting for a generic resource to load.
    /// The associated `String` represents the URL of the resource.
    WaitingForResource(ResourceType, String),
}

/// Represents the current state of the HTML parser.
#[derive(Default, Debug, Clone)]
pub enum ParserState {
    /// The parser is actively processing input.
    #[default]
    Running,

    /// The parser is blocked, waiting for a specific reason.
    Blocked(BlockedReason),

    /// The parser has completed processing.
    Completed,
}
