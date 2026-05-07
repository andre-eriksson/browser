use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Formatter},
    io::Write,
    ops::{Deref, Index, IndexMut},
};

use crate::tag::Tag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

impl From<usize> for NodeId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Deref for NodeId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    pub attributes: Option<HashMap<String, String>>,
    pub class_set: Option<HashSet<String>>,
    pub tag: Tag,
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag && self.attributes == other.attributes
    }
}

impl Default for Element {
    fn default() -> Self {
        Self {
            attributes: None,
            class_set: None,
            tag: Tag::Unknown(String::new()),
        }
    }
}

impl Element {
    /// Create a new Element with the given tag, class set, and attributes
    ///
    /// # Arguments
    /// * `tag` - The tag name of the element (e.g. div, span)
    /// * `class_set` - A set of class names for this element
    /// * `attributes` - A map of attribute names to values for this element
    #[must_use]
    pub const fn new(tag: Tag, class_set: HashSet<String>, attributes: HashMap<String, String>) -> Self {
        Self {
            attributes: Some(attributes),
            class_set: Some(class_set),
            tag,
        }
    }

    /// Get the ID attribute of this element, if present
    ///
    /// # Returns
    /// An Option containing the ID as &str, or None if not present
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.attributes
            .as_ref()
            .and_then(|attrs| attrs.get("id").map(String::as_str))
    }

    /// Get an iterator over the classes of this element
    ///
    /// # Returns
    /// An iterator over class names as &str
    pub fn classes(&self) -> impl Iterator<Item = &str> {
        self.attributes
            .as_ref()
            .and_then(|attrs| attrs.get("class"))
            .into_iter()
            .flat_map(|class_str| {
                class_str
                    .split_whitespace()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
            })
    }

    /// Check if this element has a specific attribute
    ///
    /// # Arguments
    /// * `name` - The name of the attribute to check
    ///
    /// # Returns
    /// bool indicating whether the attribute is present
    #[must_use]
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes
            .as_ref()
            .is_some_and(|attrs| attrs.contains_key(name))
    }

    /// Get the value of a specific attribute by name
    ///
    /// # Arguments
    /// * `name` - The name of the attribute to retrieve
    ///
    /// # Returns
    /// An Option containing the attribute value as &str, or None if not present
    #[must_use]
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .as_ref()
            .and_then(|attrs| attrs.get(name).map(String::as_str))
    }

    /// Get the tag name of this element as a string
    ///
    /// # Returns
    /// The tag name as &str
    #[must_use]
    pub fn tag_name(&self) -> String {
        self.tag.to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeData {
    Element(Element),
    Text(String),
}

impl NodeData {
    #[must_use]
    pub const fn as_element(&self) -> Option<&Element> {
        match self {
            Self::Element(elem) => Some(elem),
            Self::Text(_) => None,
        }
    }

    #[must_use]
    pub const fn as_text(&self) -> Option<&String> {
        match self {
            Self::Text(text) => Some(text),
            Self::Element(_) => None,
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn get_node(&self, node_id: &NodeId) -> Option<&DomNode> {
        self.nodes.get(node_id.0)
    }

    /// Walk up the DOM tree from the given node, returning all ancestor nodes
    /// (parent, grandparent, etc.) in order from nearest to farthest.
    #[must_use]
    pub fn ancestors(&self, node: &DomNode) -> Vec<&DomNode> {
        let mut result = Vec::new();
        let mut current = node.parent;

        while let Some(pid) = current {
            let parent_node = &self[pid];

            result.push(parent_node);
            current = parent_node.parent;
        }

        result
    }

    pub fn push_node(&mut self, data: &NodeData, parent: Option<NodeId>) -> NodeId {
        let node_id = NodeId(self.nodes.len());
        let new_node = DomNode {
            id: node_id,
            parent,
            children: Vec::new(),
            data: data.clone(),
        };

        self.nodes.push(new_node);

        if let Some(parent_id) = parent {
            let parent_node = &mut self[&parent_id];
            parent_node.children.push(node_id);
        } else {
            self.root_nodes.push(node_id);
        }

        node_id
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Convert the DOM tree to an HTML string representation
    /// Used for debugging and visualization purposes.
    #[must_use]
    pub fn to_html(&self) -> Vec<u8> {
        fn node_to_html(mut html: &mut Vec<u8>, node: &DomNode, dom_tree: &DocumentRoot, depth: usize) {
            if node.data.as_text().is_some_and(|t| t.trim().is_empty()) {
                return; // Skip empty text nodes
            }

            write!(&mut html, "<div class='line'>").unwrap();
            write!(&mut html, "<span style='margin-left: calc({depth} * 2rem)'></span>").unwrap();

            match &node.data {
                NodeData::Element(elem) => {
                    write!(&mut html, "<span class='tag'>&lt;</span><span class='tag-name'>{}</span>", elem.tag)
                        .unwrap();

                    write!(
                        &mut html,
                        " <span class='attr-name'>data-node-id</span><span class='attr-equals'>=</span><span class='attr-value'>\"{}\"</span>",
                        node.id,
                    ).unwrap();

                    if let Some(attrs) = &elem.attributes {
                        for (attr_name, attr_value) in attrs {
                            if attr_name.trim().is_empty() {
                                continue;
                            }

                            write!(
                                &mut html,
                                " <span class='attr-name'>{attr_name}</span><span class='attr-equals'>=</span><span class='attr-value'>\"{attr_value}\"</span>",
                            ).unwrap();
                        }
                    }

                    write!(&mut html, "<span class='tag'>&gt;</span>").unwrap();

                    let has_child = !node.children.is_empty();

                    for child_id in &node.children {
                        node_to_html(html, &dom_tree[child_id], dom_tree, depth + 1);
                    }

                    if has_child {
                        write!(&mut html, "<span style='margin-left: calc({depth} * 2rem)'></span>").unwrap();
                    }

                    if !elem.tag.is_void_element() {
                        write!(
                            &mut html,
                            "<span class='tag'>&lt;/</span><span class='tag-name'>{}</span><span class='tag'>&gt;</span>",
                            elem.tag
                        )
                        .unwrap();
                    }
                }
                NodeData::Text(text) => {
                    write!(&mut html, "<span class='text'>{text}</span>").unwrap();
                }
            }

            write!(&mut html, "</div>").unwrap();
        }

        let mut html = Vec::new();
        writeln!(&mut html, "<html><head></head><body>").unwrap();

        for root_id in &self.root_nodes {
            node_to_html(&mut html, &self[root_id], self, 0);
        }

        writeln!(&mut html, "</body></html>").unwrap();
        html
    }
}

impl Index<NodeId> for DocumentRoot {
    type Output = DomNode;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.nodes[*index]
    }
}

impl Index<&NodeId> for DocumentRoot {
    type Output = DomNode;

    fn index(&self, index: &NodeId) -> &Self::Output {
        &self.nodes[**index]
    }
}

impl IndexMut<&NodeId> for DocumentRoot {
    fn index_mut(&mut self, index: &NodeId) -> &mut Self::Output {
        &mut self.nodes[**index]
    }
}

impl Display for DocumentRoot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn fmt_node(node: &DomNode, dom_tree: &DocumentRoot, f: &mut Formatter<'_>, indent: usize) -> std::fmt::Result {
            match &node.data {
                NodeData::Element(elem) => {
                    for _ in 0..indent {
                        write!(f, " ")?;
                    }
                    write!(f, "<{} data-node-id=\"{}\"", elem.tag_name(), node.id)?;
                    if let Some(attrs) = &elem.attributes {
                        for attr in attrs {
                            if attr.0.trim().is_empty() {
                                continue;
                            }

                            write!(f, " {}=\"{}\"", attr.0, attr.1)?;
                        }
                    }
                    writeln!(f, ">")?;

                    if elem.tag.is_void_element() {
                        return Ok(());
                    }

                    for child_id in &node.children {
                        fmt_node(&dom_tree[child_id], dom_tree, f, indent + 1)?;
                    }

                    for _ in 0..indent {
                        write!(f, " ")?;
                    }

                    writeln!(f, "</{}>", elem.tag_name())?;

                    Ok(())
                }
                NodeData::Text(text) => {
                    if !text.trim().is_empty() {
                        for _ in 0..indent {
                            write!(f, " ")?;
                        }
                        writeln!(f, "{}", text.trim())?;
                    }

                    Ok(())
                }
            }
        }

        for root_id in &self.root_nodes {
            fmt_node(&self[root_id], self, f, 0)?;
        }

        Ok(())
    }
}
