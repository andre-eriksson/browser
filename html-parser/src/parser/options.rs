use std::collections::HashMap;

use shared_types::dom::SharedDomNode;

/// Represents options for the HTML parser.
/// This struct allows customization of the parser's behavior, such as enabling timers, collecting IDs and classes,
///
/// # Fields
/// * `collect_ids` - A boolean flag indicating whether to collect IDs from the HTML document.
/// * `collect_classes` - A boolean flag indicating whether to collect class names from the HTML document.
/// * `collect_external_resources` - A boolean flag indicating whether to collect external resources (e.g., scripts, stylesheets) from the HTML document.
#[derive(Default)]
pub struct ParserOptions {
    pub collect_ids: bool,
    pub collect_classes: bool,
    pub collect_external_resources: bool,
}

//// Represents metadata collected during the parsing of an HTML document.
/// This struct contains maps for IDs, classes, and external resources found in the document.
///
/// # Fields
/// * `id_map` - An optional `HashMap` mapping IDs to their corresponding DOM nodes.
/// * `class_map` - An optional `HashMap` mapping class names to vectors of DOM nodes that have those classes.
/// * `external_resources` - An optional `HashMap` mapping element names to vectors of external resource URLs associated with those elements.
pub struct ParseMetadata {
    pub id_map: Option<HashMap<String, SharedDomNode>>,
    pub class_map: Option<HashMap<String, Vec<SharedDomNode>>>,
    pub external_resources: Option<HashMap<String, Vec<String>>>,
}
