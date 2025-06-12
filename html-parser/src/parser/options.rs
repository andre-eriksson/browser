use std::collections::HashMap;

use shared_types::dom::SharedDomNode;

#[derive(Default)]
pub struct ParserOptions {
    pub timer: bool,
    pub collect_ids: bool,
    pub collect_classes: bool,
    pub collect_external_resources: bool,
    pub log_errors: bool,
}

pub struct ParseMetadata {
    pub id_map: Option<HashMap<String, SharedDomNode>>,
    pub class_map: Option<HashMap<String, Vec<SharedDomNode>>>,
}
