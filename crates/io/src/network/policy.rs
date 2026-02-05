use crate::network::policy::referrer::ReferrerPolicy;

pub mod referrer;

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
pub struct DocumentPolicy {
    pub referrer: ReferrerPolicy,
}
