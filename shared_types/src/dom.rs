use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type SharedDomNode = Rc<RefCell<DomNode>>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Element {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<SharedDomNode>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DomNode {
    Document(Vec<SharedDomNode>),
    Element(Element),
    Text(String),
    Comment(String),
    Doctype(DoctypeDeclaration),
    XmlDeclaration(XmlDeclaration),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DoctypeDeclaration {
    pub name: String, // "html"
    pub public_id: Option<String>,
    pub system_id: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct XmlDeclaration {
    pub version: String,          // "1.0"
    pub encoding: Option<String>, // "UTF-8"
    pub standalone: Option<bool>, // true/false
}
