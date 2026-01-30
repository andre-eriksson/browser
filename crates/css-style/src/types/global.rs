use crate::types::Parseable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Global {
    Inherit,
    Initial,
    Revert,
    RevertLayer,
    Unset,
}

impl Parseable for Global {
    fn parse(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "inherit" => Some(Self::Inherit),
            "initial" => Some(Self::Initial),
            "revert" => Some(Self::Revert),
            "revert-layer" => Some(Self::RevertLayer),
            "unset" => Some(Self::Unset),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_global() {
        assert_eq!(Global::parse("inherit"), Some(Global::Inherit));
        assert_eq!(Global::parse("initial"), Some(Global::Initial));
        assert_eq!(Global::parse("revert"), Some(Global::Revert));
        assert_eq!(Global::parse("revert-layer"), Some(Global::RevertLayer));
        assert_eq!(Global::parse("unset"), Some(Global::Unset));
        assert_eq!(Global::parse("unknown"), None);
    }
}
