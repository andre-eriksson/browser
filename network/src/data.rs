use std::collections::HashMap;

use crate::rules::csp::ContentSecurityPolicy;

pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub size: usize,
    pub body: String,
}

pub struct Origin {
    pub csp: ContentSecurityPolicy,
}
