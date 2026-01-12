use crate::service::network::policies::referrer::ReferrerPolicy;

#[derive(Debug, Clone, Default)]
pub struct DocumentPolicy {
    pub referrer: ReferrerPolicy,
}
