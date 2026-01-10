#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Global {
    Inherit,
    Initial,
    Revert,
    RevertLayer,
    Unset,
}

impl Global {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "inherit" => Some(Global::Inherit),
            "initial" => Some(Global::Initial),
            "revert" => Some(Global::Revert),
            "revert-layer" => Some(Global::RevertLayer),
            "unset" => Some(Global::Unset),
            _ => None,
        }
    }
}
