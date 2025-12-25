use std::collections::HashMap;

use crate::{
    dom::NodeId,
    tag::{HtmlTag, KnownTag},
};

/// Represents basic metadata about an HTML tag, including its name, attributes, and associated DOM node.
///
/// This struct is used to pass information about HTML tags to collectors during the parsing process.
pub struct TagInfo<'a> {
    /// The name of the HTML tag (e.g., `"div"`, `"span"`).
    pub tag: &'a HtmlTag,
    /// A reference to a map of attribute names and their values for the tag (e.g., `{"class": "my-class"}`).
    pub attributes: &'a HashMap<String, String>,
    /// A reference to the associated DOM node, which can be used to access the tag's position in the document structure.
    pub node_id: NodeId,
    /// Optional text data associated with the tag, only applicable for text nodes.
    pub data: Option<&'a String>,
}

/// A trait that defines a collector for metadata extracted from HTML tags during parsing.
///
/// Collectors implement this trait to gather specific information about HTML tags, such as IDs, classes, and external resources.
/// The collected metadata can then be used for various purposes, such as indexing, analysis, or further processing.
///
/// `Collector::collect` will be called for each start tag encountered during parsing and when parsing text content.
pub trait Collector {
    /// Collects metadata from the provided tag information.
    ///
    /// Will be called when building start tags.
    ///
    /// # Arguments
    /// * `tag` - A reference to a `TagInfo` struct containing the tag name, attributes, and associated DOM node.
    fn collect(&mut self, tag: &TagInfo);

    /// Converts the collected metadata into a final result.
    ///
    /// # Returns
    /// The final collected metadata.
    fn into_result(self) -> Self;
}

/// A default implementation of the `Collector` trait that collects metadata about HTML tags.
/// This implementation gathers information about IDs, classes, and external resources (like `href` and `src` attributes) from the parsed HTML tags.
#[derive(Default)]
pub struct DefaultCollector {
    /// An optional map that associates IDs with their corresponding DOM nodes.
    pub id_map: Option<HashMap<String, NodeId>>,

    /// An optional map that associates class names with vectors of DOM nodes that have those classes.
    pub class_map: Option<HashMap<String, Vec<NodeId>>>,

    /// An optional map that associates external resource URLs (like `href` and `src`) with the tags that reference them.
    pub external_resources: Option<HashMap<String, Vec<NodeId>>>,
}

impl Collector for DefaultCollector {
    fn collect(&mut self, tag: &TagInfo) {
        if tag.attributes.is_empty() {
            return;
        }

        if let Some(id_map) = &mut self.id_map
            && let Some(id) = tag.attributes.get("id")
        {
            id_map.entry(id.to_string()).or_insert(tag.node_id);
        }

        if let Some(class_map) = &mut self.class_map
            && let Some(classes) = tag.attributes.get("class")
        {
            for class in classes.split_whitespace() {
                class_map
                    .entry(class.to_string())
                    .or_default()
                    .push(tag.node_id);
            }
        }

        if let Some(external_resources) = &mut self.external_resources {
            if let Some(href) = tag.attributes.get("href") {
                if *tag.tag == HtmlTag::Known(KnownTag::A) {
                    return; // Skip anchor tags for href collection
                }

                external_resources
                    .entry(href.to_string())
                    .or_default()
                    .push(tag.node_id);
            }

            if let Some(src) = tag.attributes.get("src") {
                external_resources
                    .entry(src.to_string())
                    .or_default()
                    .push(tag.node_id);
            }
        }
    }

    fn into_result(self) -> Self {
        Self {
            id_map: self.id_map,
            class_map: self.class_map,
            external_resources: self.external_resources,
        }
    }
}
