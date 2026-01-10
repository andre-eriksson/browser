use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use crate::tag::HtmlTag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    pub attributes: HashMap<String, String>,
    pub tag: HtmlTag,
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag && self.attributes == other.attributes
    }
}

impl Default for Element {
    fn default() -> Self {
        Element {
            attributes: HashMap::new(),
            tag: HtmlTag::Unknown("".to_string()),
        }
    }
}

impl Element {
    pub fn new(tag: HtmlTag, attributes: HashMap<String, String>) -> Self {
        Element { tag, attributes }
    }

    /// Get the ID attribute of this element, if present
    ///
    /// # Returns
    /// An Option containing the ID as &str, or None if not present
    pub fn id(&self) -> Option<&str> {
        self.attributes.get("id").map(|s| s.as_str())
    }

    /// Get an iterator over the classes of this element
    ///
    /// # Returns
    /// An iterator over class names as &str
    pub fn classes(&self) -> impl Iterator<Item = &str> {
        self.attributes
            .get("class")
            .map(|s| s.split_whitespace())
            .into_iter()
            .flatten()
    }

    /// Check if this element has a specific attribute
    ///
    /// # Arguments
    /// * `name` - The name of the attribute to check
    ///
    /// # Returns
    /// bool indicating whether the attribute is present
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    /// Get the value of a specific attribute by name
    ///
    /// # Arguments
    /// * `name` - The name of the attribute to retrieve
    ///
    /// # Returns
    /// An Option containing the attribute value as &str, or None if not present
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(|s| s.as_str())
    }

    /// Get the tag name of this element as a string
    ///
    /// # Returns
    /// The tag name as &str
    pub fn tag_name(&self) -> &str {
        self.tag.as_str()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeData {
    Element(Element),
    Text(String),
}

impl NodeData {
    pub fn as_element(&self) -> Option<&Element> {
        match self {
            NodeData::Element(elem) => Some(elem),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<&String> {
        match self {
            NodeData::Text(text) => Some(text),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DomNode {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub data: NodeData,
}

#[derive(Debug, Clone, Default)]
pub struct DocumentRoot {
    pub nodes: Vec<DomNode>,
    pub root_nodes: Vec<NodeId>,
}

impl DocumentRoot {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_node(&self, node_id: &NodeId) -> Option<&DomNode> {
        self.nodes.get(node_id.0)
    }

    pub fn get_node_mut(&mut self, node_id: &NodeId) -> Option<&mut DomNode> {
        self.nodes.get_mut(node_id.0)
    }

    pub fn push_node(&mut self, data: NodeData, parent: Option<NodeId>) -> NodeId {
        let node_id = NodeId(self.nodes.len());
        let new_node = DomNode {
            id: node_id,
            parent,
            children: Vec::new(),
            data: data.clone(),
        };

        self.nodes.push(new_node);

        if let Some(parent_id) = parent {
            if let Some(parent_node) = self.get_node_mut(&parent_id) {
                parent_node.children.push(node_id);
            }
        } else {
            self.root_nodes.push(node_id);
        }

        node_id
    }
}

impl Display for DocumentRoot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn fmt_node(
            node: &DomNode,
            doc: &DocumentRoot,
            f: &mut Formatter<'_>,
            indent: usize,
        ) -> std::fmt::Result {
            for _ in 0..indent {
                write!(f, "  ")?;
            }
            match &node.data {
                NodeData::Element(elem) => {
                    writeln!(f, "<{} node_id={}>", elem.tag, node.id)?;
                    for child_id in &node.children {
                        if let Some(child_node) = doc.get_node(child_id) {
                            fmt_node(child_node, doc, f, indent + 1)?;
                        }
                    }
                    for _ in 0..indent {
                        write!(f, "  ")?;
                    }
                    writeln!(f, "</{}>", elem.tag)
                }
                NodeData::Text(text) => {
                    writeln!(f, "{}", text)
                }
            }
        }

        for root_id in &self.root_nodes {
            if let Some(root_node) = self.get_node(root_id) {
                fmt_node(root_node, self, f, 0)?;
            }
        }

        Ok(())
    }
}
