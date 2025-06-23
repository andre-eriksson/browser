use std::collections::HashMap;

use crate::dom::SharedDomNode;

/// Represents basic metadata about an HTML tag, including its name, attributes, and associated DOM node.
/// This struct is used to pass information about HTML tags to collectors during the parsing process.
///
/// # Fields
/// * `tag_name` - The name of the HTML tag (e.g., "div", "span").
/// * `attributes` - A reference to a map of attribute names and their values for the tag (e.g., `{"class": "my-class"}`).
/// * `dom_node` - A reference to a shared DOM node that represents the tag in the DOM tree. This allows collectors to access the full context of the tag within the document structure.
#[derive(Debug)]
pub struct TagInfo<'a> {
    pub tag_name: &'a str,
    pub attributes: &'a HashMap<String, String>,
    pub dom_node: &'a SharedDomNode,
}

pub trait Collector {
    /// The output type of the collector, which is returned when `into_result` is called.
    /// This type should encapsulate the metadata collected during parsing.
    /// It can be a tuple, struct, or any other type that represents the collected data.
    type Output;

    /// Collects metadata from the provided tag information.
    ///
    /// # Arguments
    /// * `tag` - A reference to a `TagInfo` struct containing the tag name, attributes, and associated DOM node.
    fn collect(&mut self, tag: &TagInfo);

    /// Converts the collected metadata into the output type defined by the `Output` associated type.
    ///
    /// # Returns
    /// The collected metadata in the form of the `Output` type.
    fn into_result(self) -> Self::Output;
}

/// A default implementation of the `Collector` trait that collects metadata about HTML tags.
/// This implementation gathers information about IDs, classes, and external resources (like `href` and `src` attributes) from the parsed HTML tags.
///
/// # Fields
/// * `id_map` - An optional map that associates IDs with their corresponding DOM nodes.
/// * `class_map` - An optional map that associates class names with vectors of DOM nodes that have those classes.
/// * `external_resources` - An optional map that associates external resource URLs (like `href` and `src`) with the tags that reference them.
#[derive(Default)]
pub struct DefaultCollector {
    pub id_map: Option<HashMap<String, SharedDomNode>>,
    pub class_map: Option<HashMap<String, Vec<SharedDomNode>>>,
    pub external_resources: Option<HashMap<String, Vec<String>>>,
}

impl Collector for DefaultCollector {
    type Output = (
        Option<HashMap<String, SharedDomNode>>,
        Option<HashMap<String, Vec<SharedDomNode>>>,
        Option<HashMap<String, Vec<String>>>,
    );

    fn collect(&mut self, tag: &TagInfo) {
        if tag.attributes.is_empty() {
            return;
        }

        if let Some(id_map) = &mut self.id_map {
            if let Some(id) = tag.attributes.get("id") {
                id_map
                    .entry(id.to_string())
                    .or_insert_with(|| tag.dom_node.clone());
            }
        }

        if let Some(class_map) = &mut self.class_map {
            if let Some(classes) = tag.attributes.get("class") {
                for class in classes.split_whitespace() {
                    class_map
                        .entry(class.to_string())
                        .or_default()
                        .push(tag.dom_node.clone());
                }
            }
        }

        if let Some(external_resources) = &mut self.external_resources {
            if let Some(href) = tag.attributes.get("href") {
                if tag.tag_name == "a" {
                    return; // Skip anchor tags for href collection
                }

                external_resources
                    .entry(href.to_string())
                    .or_default()
                    .push(tag.tag_name.to_string());
            }

            if let Some(src) = tag.attributes.get("src") {
                external_resources
                    .entry(src.to_string())
                    .or_default()
                    .push(tag.tag_name.to_string());
            }
        }
    }

    fn into_result(self) -> Self::Output {
        (self.id_map, self.class_map, self.external_resources)
    }
}
