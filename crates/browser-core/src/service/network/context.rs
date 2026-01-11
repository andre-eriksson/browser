use url::Url;

use crate::service::network::policy::referrer::ReferrerPolicy;

#[derive(Debug, Clone, Default)]
pub struct NetworkContext {
    pub document_url: Option<Url>,
    pub referrer_policy: ReferrerPolicy,
}
