#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Global {
    Inherit,
    Initial,
    Revert,
    RevertLayer,
    Unset,
}
