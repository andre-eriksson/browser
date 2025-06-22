use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// Represents a shared DOM node that can be used in a tree structure.
/// This type is a reference-counted pointer to a `RefCell<DomNode>`, allowing for mutable access to the DOM node while sharing ownership.
pub type SharedDomNode = Rc<RefCell<DomNode>>;

/// Represents an HTML element in the DOM tree.
/// It contains the tag name, attributes, and children nodes.
///
/// # Fields
/// * `tag_name` - The name of the HTML tag (e.g., "div", "span").
/// * `attributes` - A map of attribute names to their values for the element (e.g., `{"class": "my-class
/// * `children` - A vector of shared DOM nodes representing the child elements or text nodes contained within this element.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Element {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<SharedDomNode>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AtomicElement {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<AtomicDomNode>,
}

/// Represents a node in the DOM tree.
/// This enum can represent different types of nodes, including documents, elements, text nodes, comments, doctype declarations, and XML declarations.
/// Each variant corresponds to a specific type of node, allowing for a flexible and extensible representation of the DOM structure.
///
/// # Variants
/// * `Document` - Represents the root of the DOM tree, containing a vector of shared DOM nodes.
/// * `Element` - Represents an HTML element with a tag name, attributes, and children nodes.
/// * `Text` - Represents a text node containing plain text.
/// * `Comment` - Represents an HTML comment node containing the comment text.
/// * `Doctype` - Represents a doctype declaration, which includes the name, public ID, and system ID.
/// * `XmlDeclaration` - Represents an XML declaration, which includes the version, encoding, and standalone status.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DomNode {
    Document(Vec<SharedDomNode>),
    Element(Element),
    Text(String),
    Comment(String),
    Doctype(DoctypeDeclaration),
    XmlDeclaration(XmlDeclaration),
}

impl Default for DomNode {
    fn default() -> Self {
        DomNode::Document(Vec::new())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AtomicDomNode {
    Document(Vec<AtomicDomNode>),
    Element(AtomicElement),
    Text(String),
    Comment(String),
    Doctype(DoctypeDeclaration),
    XmlDeclaration(XmlDeclaration),
}

impl Default for AtomicDomNode {
    fn default() -> Self {
        AtomicDomNode::Document(Vec::new())
    }
}

impl From<&DomNode> for AtomicDomNode {
    fn from(node: &DomNode) -> Self {
        match node {
            DomNode::Document(children) => AtomicDomNode::Document(
                children
                    .iter()
                    .map(|child| AtomicDomNode::from(&*child.borrow()))
                    .collect(),
            ),
            DomNode::Element(el) => AtomicDomNode::Element(AtomicElement {
                tag_name: el.tag_name.clone(),
                attributes: el.attributes.clone(),
                children: el
                    .children
                    .iter()
                    .map(|child| AtomicDomNode::from(&*child.borrow()))
                    .collect(),
            }),
            DomNode::Text(t) => AtomicDomNode::Text(t.clone()),
            DomNode::Comment(c) => AtomicDomNode::Comment(c.clone()),
            DomNode::Doctype(d) => AtomicDomNode::Doctype(d.clone()),
            DomNode::XmlDeclaration(x) => AtomicDomNode::XmlDeclaration(x.clone()),
        }
    }
}

/// Represents a doctype declaration in the DOM tree.
/// It contains the name of the doctype, as well as optional public and system IDs.
///
/// # Fields
/// * `name` - The name of the doctype (e.g., "html").
/// * `public_id` - An optional public identifier for the doctype, which may be used to reference a specific document type definition (DTD).
/// * `system_id` - An optional system identifier for the doctype, which may be used to reference a specific resource or DTD.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DoctypeDeclaration {
    pub name: String, // "html"
    pub public_id: Option<String>,
    pub system_id: Option<String>,
}

/// Represents an XML declaration in the DOM tree.
/// It contains the version of the XML specification, an optional encoding declaration, and an optional standalone declaration.
///
/// # Fields
/// * `version` - The version of the XML specification (e.g., "1.0").
/// * `encoding` - An optional encoding declaration (e.g., "UTF-8").
/// * `standalone` - An optional boolean indicating whether the XML document is standalone (true) or not (false).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct XmlDeclaration {
    pub version: String,          // "1.0"
    pub encoding: Option<String>, // "UTF-8"
    pub standalone: Option<bool>, // true/false
}
