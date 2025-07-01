use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

// --- Multi Threaded  DOM Node ---
// Should be used in multi-threaded contexts, e.g. tokio tasks

/// Represents a multi-threaded DOM element.
///
/// # Fields
/// * `id` - A unique identifier for the element, useful for tracking and comparing elements.
/// * `tag_name` - The name of the HTML tag (e.g., "div", "span").
/// * `attributes` - A map of attribute names and their values for the element.
/// * `children` - A vector of child nodes, which can be other elements, text nodes, etc.
#[derive(Debug, Clone)]
pub struct ConcurrentElement {
    pub id: u32,
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<ArcDomNode>,
}

impl Default for ConcurrentElement {
    fn default() -> Self {
        ConcurrentElement {
            id: 0,
            tag_name: String::new(),
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }
}

/// Represents an Arc-wrapped, multi-threaded DOM node.
/// This type is used to allow shared ownership of DOM nodes in a multi-threaded context.
/// It is wrapped in a `Mutex` to allow safe concurrent access.
pub type ArcDomNode = Arc<Mutex<ConcurrentDomNode>>;

/// Represents a multi-threaded DOM node.
/// It can be a document, element, text node, comment, doctype declaration, or XML declaration.
///
/// # Variants
/// * `Document(Vec<ArcDomNode>)` - Represents the root document node containing child nodes.
/// * `Element(ConcurrentElement)` - Represents an HTML element with its attributes and children.
/// * `Text(String)` - Represents a text node containing plain text.
/// * `Comment(String)` - Represents a comment node containing comment text.
/// * `Doctype(DoctypeDeclaration)` - Represents a doctype declaration, which defines the document type.
/// * `XmlDeclaration(XmlDeclaration)` - Represents an XML declaration, which specifies the XML version
#[derive(Debug, Clone)]
pub enum ConcurrentDomNode {
    Document(Vec<ArcDomNode>),
    Element(ConcurrentElement),
    Text(String),
    Comment(String),
    Doctype(DoctypeDeclaration),
    XmlDeclaration(XmlDeclaration),
}

impl Default for ConcurrentDomNode {
    fn default() -> Self {
        ConcurrentDomNode::Document(Vec::new())
    }
}

// --- Single Threaded DOM Node ---
// Used in single-threaded contexts, e.g. parsing HTML.

/// Represents a single-threaded DOM element.
///
/// # Fields
/// * `id` - A unique identifier for the element, useful for tracking and comparing elements.
/// * `tag_name` - The name of the HTML tag (e.g., "div", "span").
/// * `attributes` - A map of attribute names and their values for the element.
/// * `children` - A vector of child nodes, which can be other elements, text nodes, etc.
#[derive(Debug, Clone)]
pub struct Element {
    pub id: u32,
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<RefDomNode>,
}

impl Default for Element {
    fn default() -> Self {
        Element {
            id: 0,
            tag_name: String::new(),
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }
}

/// Represents a single-threaded DOM node.
/// It can be a document, element, text node, comment, doctype declaration, or XML declaration.
///
/// # Variants
/// * `Document(Vec<RefDomNode>)` - Represents the root document node containing child nodes.
/// * `Element(Element)` - Represents an HTML element with its attributes and children.
/// * `Text(String)` - Represents a text node containing plain text.
/// * `Comment(String)` - Represents a comment node containing comment text.
/// * `Doctype(DoctypeDeclaration)` - Represents a doctype declaration, which defines the document type.
/// * `XmlDeclaration(XmlDeclaration)` - Represents an XML declaration, which specifies the XML version
#[derive(Debug, Clone)]
pub enum DomNode {
    Document(Vec<RefDomNode>),
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

/// Represents a reference-counted, single-threaded DOM node.
/// This type is used to allow shared ownership of DOM nodes in a single-threaded context.
pub type RefDomNode = Rc<RefCell<DomNode>>;

// --- Convert Trait ---

/// A trait which allows the implementor to convert itself into a different type.
/// Currently, it is used to convert a single-threaded DOM node into a multi-threaded one, via `ArcDomNode`.
pub trait ConvertDom<T> {
    fn convert(self) -> T;
}

impl ConvertDom<ArcDomNode> for RefDomNode {
    fn convert(self) -> ArcDomNode {
        let borrowed = self.borrow();

        let converted_node = match &*borrowed {
            DomNode::Document(children) => {
                let converted_children: Vec<ArcDomNode> = children
                    .iter()
                    .map(|child| child.clone().convert())
                    .collect();

                ConcurrentDomNode::Document(converted_children)
            }
            DomNode::Element(element) => {
                let converted_children: Vec<ArcDomNode> = element
                    .children
                    .iter()
                    .map(|child| child.clone().convert())
                    .collect();

                ConcurrentDomNode::Element(ConcurrentElement {
                    id: element.id,
                    tag_name: element.tag_name.clone(),
                    attributes: element.attributes.clone(),
                    children: converted_children,
                })
            }
            DomNode::Text(text) => ConcurrentDomNode::Text(text.clone()),
            DomNode::Comment(comment) => ConcurrentDomNode::Comment(comment.clone()),
            DomNode::Doctype(doctype) => ConcurrentDomNode::Doctype(doctype.clone()),
            DomNode::XmlDeclaration(xml_decl) => {
                ConcurrentDomNode::XmlDeclaration(xml_decl.clone())
            }
        };

        Arc::new(Mutex::new(converted_node))
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
