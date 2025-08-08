use std::collections::HashMap;

use crate::dom::{DocumentNode, SingleThreaded};

/// Represents basic metadata about an HTML tag, including its name, attributes, and associated DOM node.
/// This struct is used to pass information about HTML tags to collectors during the parsing process.
///
/// # Fields
/// * `tag_name` - The name of the HTML tag (e.g., "div", "span").
/// * `attributes` - A reference to a map of attribute names and their values for the tag (e.g., `{"class": "my-class"}`).
/// * `dom_node` - A reference to the associated DOM node, which can be used to access the tag's position in the document structure.
pub struct TagInfo<'a> {
    pub tag_name: &'a str,
    pub attributes: &'a HashMap<String, String>,
    pub dom_node: &'a DocumentNode<SingleThreaded>,
}

/// A trait that defines a collector for metadata extracted from HTML tags during parsing.
/// Collectors implement this trait to gather specific information about HTML tags, such as IDs, classes, and external resources.
/// The collected metadata can then be used for various purposes, such as indexing, analysis, or further processing.
///
/// `Collector::collect` will be called for each start tag encountered during parsing and when parsing text content.
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
    pub id_map: Option<HashMap<String, DocumentNode<SingleThreaded>>>,
    pub class_map: Option<HashMap<String, Vec<DocumentNode<SingleThreaded>>>>,
    pub external_resources: Option<HashMap<String, Vec<DocumentNode<SingleThreaded>>>>,
}

impl Collector for DefaultCollector {
    type Output = Self;

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
                    .push(tag.dom_node.clone());
            }

            if let Some(src) = tag.attributes.get("src") {
                external_resources
                    .entry(src.to_string())
                    .or_default()
                    .push(tag.dom_node.clone());
            }
        }
    }

    fn into_result(self) -> Self::Output {
        Self {
            id_map: self.id_map,
            class_map: self.class_map,
            external_resources: self.external_resources,
        }
    }
}
